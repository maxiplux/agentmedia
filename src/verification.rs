//! Previewable, content-bound, bounded live checks. No provider calls during preparation.
use crate::{
    contracts::*,
    credentials, engine,
    error::{Error, Result},
    project,
    storage::*,
    validation,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    pub schema_version: u32,
    pub id: String,
    pub description: String,
    pub jobs: Vec<PathBuf>,
    pub limits: Limits,
    pub criteria: Vec<String>,
}
pub struct Prepared {
    pub suite: Suite,
    pub inputs: Vec<Effective>,
    pub hash: String,
    pub preview: Value,
}
pub fn prepare(root: &Path, path: &Path) -> Result<Prepared> {
    let suite: Suite = yaml(path)?;
    version(suite.schema_version)?;
    id(&suite.id)?;
    validation::limits(&suite.limits)?;
    if suite.jobs.is_empty()
        || suite.jobs.len() > 32
        || suite.description.trim().is_empty()
        || suite.criteria.is_empty()
        || suite.criteria.iter().any(|c| c.trim().is_empty())
        || suite.limits.attempts != 1
    {
        return Err(Error::input(
            "Verification requires 1–32 jobs, description, acceptance criteria, and attempts: 1",
        ));
    }
    let cfg = project::config(root)?;
    let mut inputs = Vec::new();
    let base = path.parent().unwrap_or(Path::new("."));
    for source in &suite.jobs {
        let p = relative(base, source)?;
        let job: Job = yaml(&p)?;
        let input = validation::resolve_project(root, &job, p.parent().unwrap(), &cfg)?;
        if input.output.is_some() {
            return Err(Error::input(
                "Verification jobs must omit output; artifacts stay in isolated runs",
            ));
        }
        inputs.push(input);
    }
    validation::graph(&inputs)?;
    fn count(e: &Effective, b: &mut Budget) {
        if !e.turns.is_empty() {
            for t in &e.turns {
                count(t, b)
            }
            return;
        }
        b.requests += 1;
        if matches!(e.media, Media::Image | Media::Infographic) {
            b.images += 1
        }
        if e.media == Media::Speech {
            let n = e.text.as_deref().unwrap_or("").chars().count() as u32;
            b.speech_chars += n;
            b.audio_seconds += n.div_ceil(12)
        }
        if matches!(e.media, Media::Music | Media::Sfx) {
            b.audio_seconds += e.settings.duration_seconds.unwrap_or(0)
        }
    }
    let mut reservation = Budget::default();
    for e in &inputs {
        count(e, &mut reservation)
    }
    if reservation.requests > suite.limits.requests
        || reservation.images > suite.limits.images
        || reservation.audio_seconds > suite.limits.audio_seconds
        || reservation.speech_chars > suite.limits.speech_chars
    {
        return Err(Error::limit(
            "Suite exceeds its declared request/media limits",
        ));
    }
    let hash = hash(&serde_json::to_vec(&(&suite, &inputs))?);
    let preview = json!({"suite":suite,"preview_hash":hash,"estimated_reservation":reservation,"cost_estimate":null,"cost_status":"unknown; request/media limits are not a currency cap","generation_performed":false,"provider_voice_creation":false,"inputs":inputs,"next_action":"Approve this exact batch before verify --execute --preview-hash HASH. AI/human review requests are separate."});
    Ok(Prepared {
        suite,
        inputs,
        hash,
        preview,
    })
}
pub fn submit(root: &Path, p: Prepared, approved_hash: &str) -> Result<(Option<Run>, Value)> {
    if p.hash != approved_hash {
        return Err(Error::input(
            "Preview hash changed; inspect and approve a new preview",
        ));
    }
    let mut skipped = Vec::new();
    let mut blocked = BTreeSet::new();
    fn providers(e: &Effective, out: &mut BTreeSet<Provider>) {
        if e.turns.is_empty() {
            out.insert(e.selection.provider);
        } else {
            for t in &e.turns {
                providers(t, out)
            }
        }
    }
    for e in &p.inputs {
        let mut needed = BTreeSet::new();
        providers(e, &mut needed);
        for provider in needed {
            if let Err(err) = credentials::resolve(provider, root) {
                blocked.insert(e.id.clone());
                skipped.push(
                    json!({"job":e.id,"provider":provider,"status":"skipped","reason":err.code}),
                );
            }
        }
    }
    loop {
        let old = blocked.len();
        for e in &p.inputs {
            if e.depends_on.iter().any(|d| blocked.contains(d)) && blocked.insert(e.id.clone()) {
                skipped
                    .push(json!({"job":e.id,"status":"skipped","reason":"dependency unavailable"}));
            }
        }
        if old == blocked.len() {
            break;
        }
    }
    let inputs = p
        .inputs
        .into_iter()
        .filter(|e| !blocked.contains(&e.id))
        .collect::<Vec<_>>();
    if inputs.is_empty() {
        return Ok((
            None,
            json!({"skipped":skipped,"preview_hash":p.hash,"human_acceptance":"pending"}),
        ));
    }
    let run = engine::submit(
        root,
        inputs,
        p.suite.limits,
        engine::Submit {
            request_id: Some(format!("verify-{}", &p.hash[..48])),
            fresh: true,
            ..Default::default()
        },
    )?;
    let receipt = json!({"suite":p.suite.id,"preview_hash":p.hash,"criteria":p.suite.criteria,"skipped":skipped,"human_acceptance":"pending","run_id":run.id});
    let path = engine::run_path(root, &run.id)?.join("verification.yaml");
    if !path.exists() {
        save(&path, &receipt)?;
    }
    Ok((Some(run), receipt))
}
