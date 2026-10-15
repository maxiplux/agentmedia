//! Local reports and conservative, preview-bound maintenance.
use crate::{
    contracts::*,
    engine,
    error::{Error, Result},
    project,
    storage::*,
    validation,
};
use fs4::fs_std::FileExt;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn migrate(root: &Path, path: &Path, kind: &str, apply: bool) -> Result<Value> {
    let bytes = fs::read(path)?;
    let mut value: Value =
        serde_saphyr::from_slice(&bytes).map_err(|_| Error::input("Invalid manifest YAML"))?;
    let before = value["schema_version"]
        .as_u64()
        .ok_or_else(|| Error::input("Missing schema_version"))?;
    if before != 1 && before != 2 {
        return Err(Error::input("Only v1 to v2 migration is supported"));
    }
    value["schema_version"] = json!(2);
    match kind {
        "job" => {
            let job: Job = serde_json::from_value(value.clone())?;
            validation::resolve_project(
                root,
                &job,
                path.parent().unwrap_or(Path::new(".")),
                &project::config(root)?,
            )?;
        }
        "config" => validation::config(&serde_json::from_value(value.clone())?)?,
        "run" => {
            let run: Run = yaml(path)?;
            crate::storage::version2(run.schema_version)?;
            if !run.state.terminal() {
                return Err(Error::input("Cannot migrate an active run"));
            }
        }
        _ => return Err(Error::input("Migration supports job, config, or run")),
    }
    let backup = path.with_extension("v1.bak");
    if apply && before == 1 {
        let guard = if kind == "run" {
            let f = lock(&path.with_file_name("worker.lock"))?;
            if !f.try_lock_exclusive()? {
                return Err(Error::input("Run worker is active"));
            }
            Some(f)
        } else {
            None
        };
        if fs::read(path)? != bytes {
            return Err(Error::input("Manifest changed during migration"));
        }
        atomic(&backup, &bytes, false)?;
        save(path, &value)?;
        drop(guard);
    }
    Ok(
        json!({"path":absolute(path)?,"from":before,"to":2,"applied":apply&&before==1,"backup":backup,"source_sha256":hash(&bytes)}),
    )
}
pub fn usage(root: &Path) -> Result<Value> {
    let mut rows = Vec::new();
    let mut groups = std::collections::BTreeSet::new();
    let mut budgets = Vec::new();
    for run in engine::list(root)? {
        if groups.insert(run.budget_group.clone()) {
            let path = engine::run_path(root, &run.budget_group)?.join("budget.yaml");
            if path.exists() {
                budgets.push(json!({"group":run.budget_group,"reserved":yaml::<Budget>(&path)?}));
            }
        }
        for job in run.jobs {
            rows.push(json!({"run":run.id,"job":job.input.id,"selection":job.input.selection,"state":job.state,"attempts":job.attempts,"cached":job.cached,"provider_reported_usage":job.usage,"cost_estimate":null,"cost_status":"not estimated","note":"Provider usage is raw reported units; missing cost is unknown, not zero."}));
        }
    }
    let mut files = Vec::new();
    walk(&root.join("mediaagent/runs"), &mut files)?;
    let mut operations = Vec::new();
    for path in files
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "yaml"))
    {
        let name = path.file_name().unwrap().to_string_lossy();
        let operation = name == "design-operation.yaml"
            || name == "clone-operation.yaml"
            || name.starts_with("create-")
            || path
                .parent()
                .is_some_and(|p| p.file_name().is_some_and(|n| n == "operations"));
        if operation && let Ok(op) = yaml::<Operation>(path) {
            operations.push(json!({"id":op.id,"selection":op.selection,"state":op.state,"attempts":op.attempts,"provider_reported_usage":op.usage,"history":op.history.iter().map(|a|json!({"attempt":a.attempt,"state":a.state,"provider_reported_usage":a.usage})).collect::<Vec<_>>(),"cost_estimate":null}));
        }
        if name == "budget.yaml" {
            let group = path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            if groups.insert(group.clone()) {
                budgets.push(json!({"group":group,"reserved":yaml::<Budget>(path)?}));
            }
        }
    }
    Ok(
        json!({"jobs":rows,"operations":operations,"budget_groups":budgets,"currency_total":null,"currency_total_status":"unavailable; no uniform cost across providers"}),
    )
}
fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for e in fs::read_dir(dir)? {
        let e = e?;
        let t = e.file_type()?;
        if t.is_symlink() {
            return Err(Error::input("Storage maintenance refuses symbolic links"));
        }
        if t.is_dir() {
            walk(&e.path(), files)?
        } else {
            files.push(e.path())
        }
    }
    Ok(())
}
pub fn storage_plan(root: &Path) -> Result<Value> {
    let root = fs::canonicalize(root)?;
    let media = root.join("mediaagent");
    let mut files = Vec::new();
    walk(&media, &mut files)?;
    files.sort();
    let mut references = String::new();
    let mut state = Vec::new();
    let mut total = 0_u64;
    for p in &files {
        total += fs::metadata(p)?.len();
        if matches!(
            p.extension().and_then(|s| s.to_str()),
            Some("yaml" | "yml" | "md" | "bak")
        ) {
            let bytes = fs::read(p)?;
            references.push_str(&String::from_utf8_lossy(&bytes));
            state.push((p.strip_prefix(&root).unwrap().to_path_buf(), hash(&bytes)));
        }
    }
    let mut candidates = Vec::new();
    for p in &files {
        if p.parent() != Some(media.join("blobs").as_path()) {
            continue;
        }
        let name = p.file_name().unwrap().to_string_lossy();
        if crate::blobs::valid_hash(&name) && !references.contains(name.as_ref()) {
            let bytes = fs::read(p)?;
            if hash(&bytes) != name {
                return Err(Error::input("Corrupt blob; inspect before cleanup"));
            }
            candidates.push(
                json!({"path":p.strip_prefix(&root).unwrap(),"sha256":name,"bytes":bytes.len()}),
            );
        }
    }
    Ok(
        json!({"schema_version":1,"project":root,"snapshot":hash(&serde_json::to_vec(&state)?),"total_bytes":total,"files":files.len(),"candidates":candidates,"policy":"Only unreferenced hash blobs. All runs, outputs, reviews, authored assets and history are retained."}),
    )
}
pub fn clean(root: &Path, plan: &Path) -> Result<Value> {
    let expected: Value = yaml(plan)?;
    let submit = lock(&root.join("mediaagent/runs/submissions.lock"))?;
    submit.lock_exclusive()?;
    let mut workers = Vec::new();
    for run in engine::list(root)? {
        let f = lock(&engine::run_path(root, &run.id)?.join("worker.lock"))?;
        if !f.try_lock_exclusive()? || !run.state.terminal() {
            return Err(Error::input("Stop active runs before storage cleanup"));
        }
        workers.push(f);
    }
    let actual = storage_plan(root)?;
    // Exclude size/count: the saved preview itself can be inside the project.
    for key in ["project", "snapshot", "candidates"] {
        if actual[key] != expected[key] {
            return Err(Error::input(
                "Cleanup preview is stale; generate and inspect a new plan",
            ));
        }
    }
    let mut removed = Vec::new();
    for c in actual["candidates"].as_array().unwrap() {
        let p = root.join(c["path"].as_str().unwrap());
        fs::remove_file(&p)?;
        removed.push(p)
    }
    Ok(
        json!({"removed":removed,"preserved":"all run history, media outputs, authored content and references"}),
    )
}
pub fn comparison(root: &Path, ids: &[String], output: &Path) -> Result<Value> {
    if output.exists() || output.with_extension("assets").exists() {
        return Err(Error::input("Choose a new comparison report path"));
    }
    let mut items = Vec::new();
    let mut content = String::from(
        "# Media acceptance comparison\n\nAPI completion is separate from human acceptance. Compare identities, exact copy, pronunciation and scene direction against the originals.\n\n",
    );
    for id in ids {
        let mut found = None;
        for run in engine::list(root)? {
            for job in run.jobs {
                if let Some(a) = job.artifacts.iter().find(|a| a.id == *id) {
                    found = Some((a.clone(), job.input.clone(), run.id.clone()));
                }
            }
        }
        let (a, input, run) =
            found.ok_or_else(|| Error::input("Unknown comparison artifact ID"))?;
        if hash(&fs::read(&a.path)?) != a.sha256 {
            return Err(Error::input("Comparison artifact failed integrity check"));
        }
        content += &format!(
            "## {}\n\nRun: `{}`. Quality: `{:?}`. Provider/model: `{:?}` / `{}`.\n\nArtifact: <{}>\n\n",
            a.id,
            run,
            a.quality,
            input.selection.provider,
            input.selection.model,
            a.path.display()
        );
        let dir = output.with_extension("assets");
        let mut refs = Vec::new();
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        for (i, r) in input.references.iter().enumerate() {
            let bytes = STANDARD
                .decode(&r.base64)
                .map_err(|_| Error::input("Invalid reference"))?;
            if hash(&bytes) != r.sha256 {
                return Err(Error::input("Reference hash mismatch"));
            }
            let ext = match r.mime.as_str() {
                "image/png" => "png",
                "image/jpeg" => "jpg",
                "image/webp" => "webp",
                "audio/wav" => "wav",
                "audio/mpeg" => "mp3",
                _ => "flac",
            };
            let path = dir.join(format!("{}-{i}.{ext}", a.id));
            create(&path, &bytes)?;
            let path = absolute(&path)?;
            content += &format!(
                "Reference {} ({:?}): <{}>\n\n",
                i + 1,
                r.role,
                path.display()
            );
            refs.push(path);
        }
        content += "| Generated asset | Original references |\n| --- | --- |\n";
        let generated = if matches!(a.format.as_str(), "png" | "jpeg" | "webp" | "wav" | "mp3") {
            format!("![Generated](<{}>)", a.path.display())
        } else {
            format!("[Generated](<{}>)", a.path.display())
        };
        let reference_previews = refs
            .iter()
            .enumerate()
            .map(|(i, p)| format!("![Reference {}](<{}>)", i + 1, p.display()))
            .collect::<Vec<_>>()
            .join(" ");
        content += &format!("| {generated} | {reference_previews} |\n\n");
        content += "Review criteria: identity / exact copy / readability / pronunciation / timing / clipping. Record findings with `review record`; this report does not accept anything.\n\n";
        let reviews_path = engine::run_path(root, &run)?.join("reviews");
        let mut reviews = Vec::new();
        if reviews_path.exists() {
            for entry in fs::read_dir(reviews_path)? {
                let path = entry?.path();
                if path.extension().is_some_and(|e| e == "yaml") {
                    let review: Review = yaml(&path)?;
                    if review.artifact == a.id {
                        content += &format!(
                            "Recorded review: {} / {:?}.\n\n",
                            review.provenance, review.quality
                        );
                        for f in &review.findings {
                            content += &format!(
                                "- {}: expected {}; observed {}; evidence {}.\n",
                                f.check.replace('\n', " "),
                                f.expected.replace('\n', " "),
                                f.observed.replace('\n', " "),
                                f.evidence.replace('\n', " ")
                            );
                        }
                        content.push('\n');
                        reviews.push(review);
                    }
                }
            }
        }
        items.push(json!({"artifact":a,"references":refs,"run":run,"reviews":reviews}));
    }
    atomic(output, content.as_bytes(), false)?;
    Ok(json!({"report":absolute(output)?,"items":items,"human_acceptance":"pending"}))
}
