use crate::{
    audio,
    contracts::*,
    error::{Error, Result},
    providers::{Adapter, Generated},
    storage::*,
    validation,
};
use fs4::fs_std::FileExt;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Semaphore;
use uuid::Uuid;

pub fn run_path(root: &Path, id: &str) -> Result<PathBuf> {
    crate::storage::id(id)?;
    Ok(root.join("mediaagent/runs").join(id))
}
pub fn load(root: &Path, id: &str) -> Result<Run> {
    let run: Run = yaml(&run_path(root, id)?.join("run.yaml"))?;
    version2(run.schema_version)?;
    Ok(run)
}
pub fn list(root: &Path) -> Result<Vec<Run>> {
    let path = root.join("mediaagent/runs");
    let mut runs: Vec<Run> = vec![];
    if path.exists() {
        for entry in fs::read_dir(path)? {
            let p = entry?.path().join("run.yaml");
            if p.is_file() {
                runs.push(yaml(&p)?);
            }
        }
    }
    runs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(runs)
}
pub fn status(root: &Path, id: &str) -> Result<Run> {
    let run = load(root, id)?;
    if run.state.terminal() {
        return Ok(run);
    }
    status_after_observation(root, id, run)
}
fn status_after_observation(root: &Path, id: &str, mut run: Run) -> Result<Run> {
    let path = run_path(root, id)?;
    let f = lock(&path.join("worker.lock"))?;
    if !f.try_lock_exclusive()? {
        return Ok(run);
    }
    // The worker can finish after our first read and before this lock is acquired.
    // Never overwrite its terminal receipt with an orphaned stale snapshot.
    run = load(root, id)?;
    if run.state.terminal() {
        return Ok(run);
    }
    if run.state == State::Queued
        && fs::metadata(path.join("run.yaml"))?
            .modified()?
            .elapsed()
            .unwrap_or_default()
            < Duration::from_secs(10)
    {
        return Ok(run);
    }
    for j in &mut run.jobs {
        if j.state == State::Running {
            j.state = if !j.input.turns.is_empty() {
                State::Queued
            } else {
                State::Unknown
            };
            j.error = Some("outcome_unknown: worker disappeared during submission".into());
        }
    }
    run.state = if run.jobs.iter().any(|j| j.state == State::Unknown) {
        State::Unknown
    } else {
        State::Blocked
    };
    save(&path.join("run.yaml"), &run)?;
    Ok(run)
}
#[derive(Default)]
pub struct Submit {
    pub request_id: Option<String>,
    pub fresh: bool,
    pub overwrite: bool,
    pub output_dir: Option<PathBuf>,
    pub parent_artifact: Option<String>,
    pub budget_group: Option<String>,
}
pub fn submit(root: &Path, inputs: Vec<Effective>, limits: Limits, options: Submit) -> Result<Run> {
    validation::graph(&inputs)?;
    for input in &inputs {
        crate::providers::validate(input)?;
    }
    validation::limits(&limits)?;
    let submission_fingerprint = hash(&serde_json::to_vec(&(
        &inputs,
        &options.output_dir,
        options.fresh,
        options.overwrite,
        &limits,
    ))?);
    let submit_lock = lock(&root.join("mediaagent/runs/submissions.lock"))?;
    submit_lock.lock_exclusive()?;
    if let Some(key) = &options.request_id {
        crate::storage::id(key)?;
        for run in list(root)? {
            if run.request_id.as_ref() == Some(key) {
                if run.submission_fingerprint != submission_fingerprint {
                    return Err(Error::input(
                        "request-id already belongs to different effective input",
                    ));
                }
                return Ok(run);
            }
        }
    }
    let id = Uuid::new_v4().to_string();
    let path = run_path(root, &id)?;
    fs::create_dir_all(path.join("outputs"))?;
    let mut jobs = vec![];
    fn expand(
        mut input: Effective,
        jobs: &mut Vec<JobRecord>,
        output_dir: Option<&Path>,
    ) -> Result<()> {
        if !input.turns.is_empty() {
            let dependencies = input.depends_on.clone();
            for turn in &input.turns {
                let mut child = turn.clone();
                child.id = format!("{}--{}", input.id, child.id);
                crate::storage::id(&child.id)?;
                child.depends_on = dependencies.clone();
                input.depends_on.push(child.id.clone());
                expand(child, jobs, output_dir)?;
            }
        }
        if let Some(dir) = output_dir {
            input.output = Some(absolute(dir)?.join(format!("{}.{}", input.id, format(&input))));
        }
        jobs.push(record(input)?);
        Ok(())
    }
    for input in inputs {
        if input.id.contains("--") {
            return Err(Error::input("Job IDs cannot contain reserved -- separator"));
        }
        expand(input, &mut jobs, options.output_dir.as_deref())?;
    }
    // Turn destinations follow the same output directory, while their originals remain in the run.
    for j in &mut jobs {
        if j.input.id.contains("--")
            && let Some(d) = &options.output_dir
        {
            j.input.output = Some(absolute(d)?.join(format!("{}.wav", j.input.id)));
        }
    }
    let run = Run {
        schema_version: 2,
        id: id.clone(),
        budget_group: options.budget_group.unwrap_or(id),
        request_id: options.request_id,
        state: State::Queued,
        jobs,
        limits,
        fresh: options.fresh,
        overwrite: options.overwrite,
        parent_artifact: options.parent_artifact,
        submission_fingerprint,
    };
    validation::graph(&run.jobs.iter().map(|j| j.input.clone()).collect::<Vec<_>>())?;
    for job in &run.jobs {
        if job.input.output.as_ref().is_some_and(|p| p.exists()) && !run.overwrite {
            return Err(Error::input(
                "Output already exists; use --overwrite explicitly",
            ));
        }
    }
    save(&path.join("run.yaml"), &run)?;
    for j in &run.jobs {
        crate::progress::save(&path, &j.input.id, crate::progress::Stage::Queued)?;
    }
    Ok(run)
}
fn record(input: Effective) -> Result<JobRecord> {
    let fingerprint = validation::fingerprint(&input)?;
    Ok(JobRecord {
        input,
        fingerprint,
        state: State::Queued,
        attempts: 0,
        artifacts: vec![],
        error: None,
        request_id: None,
        usage: None,
        cached: false,
        retry_not_before: None,
        duration_accounted: false,
    })
}
fn format(input: &Effective) -> &str {
    match input.media {
        Media::Image | Media::Infographic => "png",
        Media::Speech | Media::Podcast => "wav",
        Media::Music | Media::Sfx => "mp3",
        Media::Text => "md",
    }
}
pub fn reserve(
    root: &Path,
    group: &str,
    l: &Limits,
    e: Option<&Effective>,
    revision: bool,
) -> Result<()> {
    let path = run_path(root, group)?;
    let f = lock(&path.join("budget.lock"))?;
    f.lock_exclusive()?;
    let p = path.join("budget.yaml");
    let mut b: Budget = if p.exists() {
        yaml(&p)?
    } else {
        Budget::default()
    };
    if revision {
        b.revisions += 1;
    }
    if let Some(e) = e {
        b.requests += 1;
        if matches!(e.media, Media::Image | Media::Infographic) {
            b.images += 1;
        }
        if e.media == Media::Speech {
            let chars = e.text.as_deref().unwrap_or("").chars().count() as u32;
            b.speech_chars += chars;
            b.audio_seconds += chars.div_ceil(12);
        }
        if matches!(e.media, Media::Music | Media::Sfx) {
            b.audio_seconds += e.settings.duration_seconds.unwrap_or(0);
        }
    }
    if b.requests > l.requests
        || b.images > l.images
        || b.audio_seconds > l.audio_seconds
        || b.speech_chars > l.speech_chars
        || b.revisions > l.revisions
    {
        return Err(Error::limit(
            "Budget group exhausted; inspect budget.yaml and explicitly revise limits before more generation",
        ));
    }
    save(&p, &b)
}
fn actual_duration(
    root: &Path,
    group: &str,
    l: &Limits,
    input: &Effective,
    actual: f64,
) -> Result<bool> {
    let reserved = if input.media == Media::Speech {
        (input.text.as_deref().unwrap_or("").chars().count() as u32).div_ceil(12)
    } else {
        input.settings.duration_seconds.unwrap_or(0)
    };
    let extra = (actual.ceil() as u32).saturating_sub(reserved);
    if extra == 0 {
        return Ok(false);
    }
    let path = run_path(root, group)?;
    let f = lock(&path.join("budget.lock"))?;
    f.lock_exclusive()?;
    let p = path.join("budget.yaml");
    let mut b: Budget = yaml(&p)?;
    b.audio_seconds = b.audio_seconds.saturating_add(extra);
    save(&p, &b)?;
    Ok(b.audio_seconds > l.audio_seconds)
}
pub(crate) async fn slots(provider: Provider) -> Result<(fs::File, fs::File)> {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "local".into());
    let dir =
        std::env::temp_dir().join(format!("mediaagent-slots-{}", &hash(user.as_bytes())[..16]));
    loop {
        let limits = crate::setup::host()?;
        if let Some(slots) = try_slots(&dir, provider, &limits)? {
            return Ok(slots);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn try_slots(
    dir: &Path,
    provider: Provider,
    limits: &crate::setup::HostLimits,
) -> Result<Option<(fs::File, fs::File)>> {
    crate::setup::validate_host(limits)?;
    let allocator = lock(&dir.join("allocator.lock"))?;
    if allocator.try_lock_exclusive()? {
        let mut globals = vec![];
        let mut providers = vec![];
        for index in 0..32 {
            let file = lock(&dir.join(format!("global-{index}.lock")))?;
            if file.try_lock_exclusive()? {
                globals.push(file);
            }
        }
        for index in 0..16 {
            let file = lock(&dir.join(format!("{provider:?}-{index}.lock")))?;
            if file.try_lock_exclusive()? {
                providers.push(file);
            }
        }
        // Count all active slots, including jobs admitted under older, higher ceilings.
        if 32 - globals.len() < limits.concurrency && 16 - providers.len() < limits.per_provider {
            let global = globals.remove(0);
            let specific = providers.remove(0);
            return Ok(Some((global, specific)));
        }
    }
    Ok(None)
}

fn verified(a: &Artifact) -> bool {
    fs::read(&a.path).is_ok_and(|bytes| bytes.len() as u64 == a.bytes && hash(&bytes) == a.sha256)
}
fn cached(root: &Path, fingerprint: &str) -> Result<Option<JobRecord>> {
    let runs = root.join("mediaagent/runs");
    if !runs.exists() {
        return Ok(None);
    }
    let mut paths = fs::read_dir(runs)?
        .map(|e| e.map(|e| e.path().join("run.yaml")))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        if !path.is_file() {
            continue;
        }
        let meta = crate::blobs::read_metadata(&path)?;
        if !meta["jobs"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|j| j["fingerprint"] == fingerprint && j["state"] == "succeeded")
        {
            continue;
        }
        let run: Run = yaml(&path)?;
        if let Some(j) = run.jobs.into_iter().find(|j| {
            j.fingerprint == fingerprint
                && j.state == State::Succeeded
                && !j.artifacts.is_empty()
                && j.artifacts.iter().all(verified)
        }) {
            return Ok(Some(j));
        }
    }
    Ok(None)
}
pub fn integrity(bytes: &[u8], format: &str) -> Result<Option<f64>> {
    if format == "md" {
        if bytes.is_empty() || std::str::from_utf8(bytes).is_err() {
            return Err(Error::new(
                "invalid_text",
                "Provider text is empty or invalid UTF-8",
                3,
            ));
        }
        return Ok(None);
    }
    if ["png", "jpeg", "jpg", "webp"].contains(&format) {
        if format == "png" && !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err(Error::new(
                "invalid_image",
                "Provider did not return the requested PNG format",
                3,
            ));
        }
        let image = image::load_from_memory(bytes)
            .map_err(|_| Error::new("invalid_image", "Provider image cannot be decoded", 3))?;
        if image.width() == 0 || image.height() == 0 {
            return Err(Error::new("invalid_image", "Image has no dimensions", 3));
        }
        return Ok(None);
    }
    let (samples, rate) = audio::decode(bytes)?;
    Ok(Some(samples.len() as f64 / rate as f64))
}
pub(crate) fn artifact(
    path: &Path,
    job_id: &str,
    bytes: &[u8],
    fmt: &str,
    parent: Option<String>,
) -> Result<Artifact> {
    let dimensions = if ["png", "jpeg", "jpg", "webp"].contains(&fmt) {
        Some(
            image::ImageReader::new(std::io::Cursor::new(bytes))
                .with_guessed_format()?
                .into_dimensions()
                .map_err(|_| Error::new("invalid_image", "Cannot read image dimensions", 3))?,
        )
    } else {
        None
    };
    Ok(Artifact {
        id: Uuid::new_v4().to_string(),
        job_id: job_id.into(),
        path: absolute(path)?,
        sha256: hash(bytes),
        bytes: bytes.len() as u64,
        format: fmt.into(),
        quality: Quality::Unchecked,
        parent,
        duration_seconds: integrity(bytes, fmt)?,
        width: dimensions.map(|d| d.0),
        height: dimensions.map(|d| d.1),
    })
}
fn publish(path: &Path, run: &Run, job: &mut JobRecord, bytes: &[u8], fmt: &str) -> Result<()> {
    let output = path
        .join("outputs")
        .join(format!("{}.{}", job.input.id, fmt));
    let a = artifact(
        &output,
        &job.input.id,
        bytes,
        fmt,
        run.parent_artifact.clone(),
    )?;
    atomic(&output, bytes, true)?;
    job.artifacts.push(a);
    if let Some(dest) = &job.input.output {
        atomic(dest, bytes, run.overwrite)?;
        job.artifacts.last_mut().unwrap().path = absolute(dest)?;
    }
    Ok(())
}
#[derive(serde::Serialize, serde::Deserialize)]
struct ResponseMeta {
    format: String,
    request_id: Option<String>,
    usage: Option<serde_json::Value>,
}
async fn execute(
    root: PathBuf,
    path: PathBuf,
    run: Run,
    mut job: JobRecord,
    adapter: Arc<dyn Adapter>,
    semaphore: Arc<Semaphore>,
) -> JobRecord {
    let result:Result<()>=async {
        let _permit=semaphore.acquire_owned().await.map_err(|_|Error::new("worker","Scheduler stopped",3))?;
        let _slots=slots(job.input.selection.provider).await?;
        crate::progress::save(&path,&job.input.id,crate::progress::Stage::Generating)?;
        let raw=path.join("responses").join(format!("{}.bin",job.input.id));let meta=raw.with_extension("yaml");
        let mut generated=if raw.exists()&&meta.exists() {
            let m:ResponseMeta=yaml(&meta)?; Generated {bytes:fs::read(&raw)?,format:m.format,request_id:m.request_id,usage:m.usage}
        } else {
            let result=adapter.generate(&job.input).await?;
            atomic(&raw,&result.bytes,true)?;save(&meta,&ResponseMeta {format:result.format.clone(),request_id:result.request_id.clone(),usage:result.usage.clone()})?;
            result
        };
        if matches!(job.input.media,Media::Image|Media::Infographic) {
            generated.format=match image::guess_format(&generated.bytes).map_err(|_|Error::new("invalid_image","Cannot identify provider image format",3))? {image::ImageFormat::Png=>"png",image::ImageFormat::Jpeg=>"jpeg",image::ImageFormat::WebP=>"webp",_=>return Err(Error::new("invalid_image","Unsupported provider image format",3))}.into();
        }
        job.request_id=generated.request_id;job.usage=generated.usage;
        integrity(&generated.bytes,&generated.format)?;
        // Keep the original provider clip; assembled speech always has the stable local format.
        let (bytes,fmt)=if job.input.media==Media::Speech {
            crate::progress::save(&path,&job.input.id,crate::progress::Stage::Assembling)?;
            let original=path.join("outputs").join(format!("{}-original.{}",job.input.id,generated.format));atomic(&original,&generated.bytes,true)?;
            job.artifacts.push(artifact(&original,&job.input.id,&generated.bytes,&generated.format,run.parent_artifact.clone())?);
            (audio::wav(&audio::mono_24k(&generated.bytes)?)?,"wav".to_owned())
        } else if matches!(job.input.media,Media::Image|Media::Infographic)&&generated.format!="png" {
            let original=path.join("outputs").join(format!("{}-original.{}",job.input.id,generated.format));atomic(&original,&generated.bytes,true)?;
            job.artifacts.push(artifact(&original,&job.input.id,&generated.bytes,&generated.format,run.parent_artifact.clone())?);
            let decoded=image::load_from_memory(&generated.bytes).map_err(|_|Error::new("invalid_image","Cannot decode provider image",3))?;
            let mut buffer=std::io::Cursor::new(vec![]);decoded.write_to(&mut buffer,image::ImageFormat::Png).map_err(|_|Error::new("image_write","Cannot normalize image to PNG",3))?;
            (buffer.into_inner(),"png".into())
        } else {(generated.bytes,generated.format)};
        crate::progress::save(&path,&job.input.id,crate::progress::Stage::Saving)?;
        publish(&path,&run,&mut job,&bytes,&fmt)?;
        if !job.duration_accounted && let Some(actual)=job.artifacts.last().and_then(|a|a.duration_seconds) {
            let overrun=actual_duration(&root,&run.budget_group,&run.limits,&job.input,actual)?;job.duration_accounted=true;
            if overrun {return Err(Error::limit("Actual audio exceeded the reserved duration; artifact saved, further requests blocked"));}
        }
        Ok(())
    }.await;
    match result {
        Ok(()) => {
            job.state = State::Succeeded;
            job.error = None;
        }
        Err(e) => {
            job.state = if e.exit == 7 {
                State::Unknown
            } else if e.exit == 6 {
                State::Blocked
            } else {
                State::Failed
            };
            job.error = Some(e.to_string());
            job.retry_not_before = e.retry_after_seconds.map(|s| epoch().saturating_add(s));
            if e.request_id.is_some() {
                job.request_id = e.request_id;
            }
        }
    }
    let _ = crate::progress::save(
        &path,
        &job.input.id,
        if job.state == State::Succeeded {
            crate::progress::Stage::Complete
        } else {
            crate::progress::Stage::Failed
        },
    );
    job
}
pub async fn work(
    root: &Path,
    id: &str,
    adapter: Arc<dyn Adapter>,
    retry_unknown: bool,
) -> Result<Run> {
    let path = run_path(root, id)?;
    let guard = lock(&path.join("worker.lock"))?;
    if !guard.try_lock_exclusive()? {
        return Err(Error::new(
            "worker_active",
            "A worker already owns this run",
            3,
        ));
    }
    let mut run = load(root, id)?;
    for j in &mut run.jobs {
        if j.state == State::Succeeded {
            if !j.artifacts.iter().all(verified) {
                return Err(Error::new(
                    "artifact_corrupt",
                    "Successful artifact no longer verifies; use a new fresh run",
                    3,
                ));
            }
            continue;
        }
        if j.state == State::Running {
            j.state = if !j.input.turns.is_empty() {
                State::Queued
            } else {
                State::Unknown
            };
            j.error = Some("Worker disappeared during submission".into());
        }
        let raw = path.join("responses").join(format!("{}.bin", j.input.id));
        let recoverable = raw.exists() && raw.with_extension("yaml").exists();
        if j.state == State::Unknown && !retry_unknown && !recoverable {
            continue;
        }
        if matches!(
            j.state,
            State::Unknown | State::Failed | State::Blocked | State::Cancelled
        ) {
            j.state = State::Queued;
            j.error = None;
        }
    }
    run.state = State::Running;
    save(&path.join("run.yaml"), &run)?;
    let mut handles = tokio::task::JoinSet::new();
    let global = Arc::new(Semaphore::new(run.limits.concurrency));
    let mut semaphores = BTreeMap::new();
    for p in [
        Provider::Openai,
        Provider::Google,
        Provider::Fish,
        Provider::Openrouter,
        Provider::Elevenlabs,
    ] {
        semaphores.insert(p, Arc::new(Semaphore::new(run.limits.per_provider)));
    }
    loop {
        let cancel = path.join("cancel").exists();
        let states: BTreeMap<_, _> = run
            .jobs
            .iter()
            .map(|j| (j.input.id.clone(), j.state))
            .collect();
        let mut launch = vec![];
        let mut running = run
            .jobs
            .iter()
            .filter(|j| j.state == State::Running)
            .count();
        let mut provider_running: BTreeMap<Provider, usize> = BTreeMap::new();
        for j in run.jobs.iter().filter(|j| j.state == State::Running) {
            *provider_running
                .entry(j.input.selection.provider)
                .or_default() += 1;
        }
        for i in 0..run.jobs.len() {
            if run.jobs[i].state != State::Queued {
                continue;
            }
            if cancel {
                run.jobs[i].state = State::Cancelled;
                continue;
            }
            let deps = &run.jobs[i].input.depends_on;
            if deps.iter().any(|d| {
                states
                    .get(d)
                    .is_some_and(|s| s.terminal() && *s != State::Succeeded)
            }) {
                run.jobs[i].state = State::Blocked;
                run.jobs[i].error = Some("Dependency did not succeed".into());
                continue;
            }
            if !deps
                .iter()
                .all(|d| states.get(d) == Some(&State::Succeeded))
            {
                continue;
            }
            if run.jobs[i].input.media == Media::Podcast
                || (run.jobs[i].input.media == Media::Speech && !run.jobs[i].input.turns.is_empty())
            {
                run.jobs[i].state = State::Running;
                crate::progress::save(
                    &path,
                    &run.jobs[i].input.id,
                    crate::progress::Stage::Assembling,
                )?;
                save(&path.join("run.yaml"), &run)?;
                let mut j = run.jobs[i].clone();
                let result: Result<()> = (|| {
                    let mut clips = vec![];
                    for turn in &j.input.turns {
                        let name = format!("{}--{}", j.input.id, turn.id);
                        let clip = run
                            .jobs
                            .iter()
                            .find(|record| record.input.id == name)
                            .and_then(|record| record.artifacts.last())
                            .ok_or_else(|| {
                                Error::new("assembly", "A successful turn has no artifact", 3)
                            })?;
                        clips.push((turn.id.clone(), audio::mono_24k(&fs::read(&clip.path)?)?));
                    }
                    let combined = audio::assemble(&clips, j.input.assembly.as_ref())?;
                    publish(&path, &run, &mut j, &audio::wav(&combined)?, "wav")
                })();
                match result {
                    Ok(()) => {
                        j.state = State::Succeeded;
                        j.error = None;
                    }
                    Err(e) => {
                        j.state = State::Failed;
                        j.error = Some(e.to_string());
                    }
                }
                crate::progress::save(
                    &path,
                    &j.input.id,
                    if j.state == State::Succeeded {
                        crate::progress::Stage::Complete
                    } else {
                        crate::progress::Stage::Failed
                    },
                )?;
                run.jobs[i] = j;
                continue;
            }
            if !run.fresh
                && let Some(previous) = cached(root, &run.jobs[i].fingerprint)?
            {
                let mut j = run.jobs[i].clone();
                let a = previous.artifacts.last().unwrap();
                match fs::read(&a.path)
                    .map_err(Error::io)
                    .and_then(|bytes| publish(&path, &run, &mut j, &bytes, &a.format))
                {
                    Ok(()) => {
                        j.state = State::Succeeded;
                        j.cached = true;
                        j.error = None;
                    }
                    Err(error) => {
                        j.state = State::Failed;
                        j.error = Some(error.to_string());
                    }
                }
                crate::progress::save(
                    &path,
                    &j.input.id,
                    if j.state == State::Succeeded {
                        crate::progress::Stage::Complete
                    } else {
                        crate::progress::Stage::Failed
                    },
                )?;
                run.jobs[i] = j;
                continue;
            }
            let provider = run.jobs[i].input.selection.provider;
            if running >= run.limits.concurrency
                || provider_running.get(&provider).copied().unwrap_or(0) >= run.limits.per_provider
            {
                continue;
            }
            if let Some(next) = run.jobs[i].retry_not_before
                && next > epoch()
            {
                tokio::time::sleep(Duration::from_secs(next.saturating_sub(epoch()).min(1))).await;
                continue;
            }
            let raw = path
                .join("responses")
                .join(format!("{}.bin", run.jobs[i].input.id));
            if run.jobs[i].attempts >= run.limits.attempts
                && !(raw.exists() && raw.with_extension("yaml").exists())
            {
                run.jobs[i].state = State::Blocked;
                run.jobs[i].error = Some("limit_exceeded: attempt limit exhausted".into());
                continue;
            }
            if !(raw.exists() && raw.with_extension("yaml").exists()) {
                if let Err(error) = crate::catalog::require_input(&run.jobs[i].input) {
                    run.jobs[i].state = State::Blocked;
                    run.jobs[i].error = Some(error.to_string());
                    continue;
                }
                if let Err(e) = reserve(
                    root,
                    &run.budget_group,
                    &run.limits,
                    Some(&run.jobs[i].input),
                    false,
                ) {
                    run.jobs[i].state = State::Blocked;
                    run.jobs[i].error = Some(e.to_string());
                    continue;
                }
                run.jobs[i].attempts += 1;
            }
            run.jobs[i].state = State::Running;
            launch.push(i);
            running += 1;
            *provider_running.entry(provider).or_default() += 1;
        }
        save(&path.join("run.yaml"), &run)?;
        for i in launch {
            let root = root.to_owned();
            let path = path.clone();
            let snapshot = run.clone();
            let job = run.jobs[i].clone();
            let adapter = adapter.clone();
            let p = semaphores[&job.input.selection.provider].clone();
            let g = global.clone();
            handles.spawn(async move {
                let _g = g.acquire_owned().await.unwrap();
                (i, execute(root, path, snapshot, job, adapter, p).await)
            });
        }
        if handles.is_empty() {
            let queued = run.jobs.iter().any(|j| j.state == State::Queued);
            let changed = run
                .jobs
                .iter()
                .any(|j| states.get(&j.input.id) != Some(&j.state));
            let delayed = run.jobs.iter().any(|j| {
                j.state == State::Queued && j.retry_not_before.is_some_and(|t| t > epoch())
            });
            if queued && (changed || delayed) {
                continue;
            }
            break;
        }
        if let Some(result) = handles.join_next().await {
            let (i, job) = result.map_err(|_| {
                Error::unknown(
                    "Worker task stopped unexpectedly; running requests require reconciliation",
                )
            })?;
            run.jobs[i] = job;
        }
    }
    run.state = if run.jobs.iter().any(|j| j.state == State::Unknown) {
        State::Unknown
    } else if run.jobs.iter().any(|j| {
        j.state == State::Blocked
            && j.error
                .as_ref()
                .is_some_and(|e| e.contains("limit_exceeded"))
    }) {
        State::Blocked
    } else if run.jobs.iter().all(|j| j.state == State::Succeeded) {
        State::Succeeded
    } else if run.jobs.iter().any(|j| j.state == State::Succeeded) {
        State::Partial
    } else if run.jobs.iter().any(|j| j.state == State::Cancelled) {
        State::Cancelled
    } else if run.jobs.iter().any(|j| j.state == State::Blocked) {
        State::Blocked
    } else {
        State::Failed
    };
    save(&path.join("run.yaml"), &run)?;
    Ok(run)
}
fn epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn exit(run: &Run) -> u8 {
    match run.state {
        State::Succeeded | State::Queued | State::Running | State::Cancelling => 0,
        State::Unknown => 7,
        State::Blocked
            if run.jobs.iter().any(|j| {
                j.error
                    .as_ref()
                    .is_some_and(|e| e.contains("limit_exceeded"))
            }) =>
        {
            6
        }
        State::Partial => 4,
        _ => 3,
    }
}
pub fn spawn(root: &Path, id: &str, retry_unknown: bool) -> Result<()> {
    let path = run_path(root, id)?;
    let guard = lock(&path.join("worker.lock"))?;
    if !guard.try_lock_exclusive()? {
        return Ok(());
    }
    let mut run = load(root, id)?;
    if run.state == State::Succeeded {
        return Ok(());
    }
    run.state = State::Queued;
    save(&path.join("run.yaml"), &run)?;
    if path.join("cancel").exists() {
        fs::remove_file(path.join("cancel"))?;
    }
    let mut command = std::process::Command::new(std::env::current_exe()?);
    command
        .arg("--project")
        .arg(root)
        .arg("worker")
        .arg(id)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if retry_unknown {
        command.arg("--retry-unknown");
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x00000008 | 0x00000200);
    }
    drop(guard);
    match command.spawn() {
        Ok(mut child) => {
            let start = std::time::Instant::now();
            loop {
                // A parent lock probe can make the child fail its own ownership check.
                // Only the owning worker changes queued to running (or a terminal state).
                if load(root, id)?.state != State::Queued {
                    return Ok(());
                }
                if child.try_wait()?.is_some() || start.elapsed() > Duration::from_secs(5) {
                    let mut error = Error::new(
                        "worker_start",
                        format!(
                            "Worker did not confirm startup for run {id}; inspect the saved run before resuming"
                        ),
                        3,
                    );
                    error.corrective_argv = vec![
                        "mediaagent".into(),
                        "--project".into(),
                        root.to_string_lossy().into_owned(),
                        "runs".into(),
                        "status".into(),
                        id.into(),
                        "--json".into(),
                    ];
                    return Err(error);
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        Err(e) => {
            run.state = State::Failed;
            save(&path.join("run.yaml"), &run)?;
            Err(e.into())
        }
    }
}
pub fn cancel(root: &Path, id: &str) -> Result<Run> {
    let path = run_path(root, id)?;
    let run = load(root, id)?;
    if run.state.terminal() {
        return Ok(run);
    }
    atomic(&path.join("cancel"), b"cancel", true)?;
    let mut result = run;
    result.state = State::Cancelling;
    Ok(result)
}

/// Retrieve provider results without submitting or reserving another generation.
pub async fn reconcile(root: &Path, id: &str, adapter: &dyn Adapter) -> Result<Run> {
    let path = run_path(root, id)?;
    let guard = lock(&path.join("worker.lock"))?;
    if !guard.try_lock_exclusive()? {
        return Err(Error::new(
            "worker_active",
            "Wait for the current worker before provider reconciliation",
            3,
        ));
    }
    let mut run = load(root, id)?;
    for job in &mut run.jobs {
        if job.state == State::Running {
            job.state = if job.input.turns.is_empty() {
                State::Unknown
            } else {
                State::Queued
            };
        }
        if job.state != State::Unknown {
            continue;
        }
        let Some(remote) = job.request_id.as_deref() else {
            continue;
        };
        if job.input.selection.provider != Provider::Google {
            continue;
        }
        let _slots = slots(Provider::Google).await?;
        match adapter.reconcile(&job.input, remote).await {
            Ok(crate::providers::Reconciled::Complete(result)) => {
                let raw = path.join("responses").join(format!("{}.bin", job.input.id));
                atomic(&raw, &result.bytes, true)?;
                save(
                    &raw.with_extension("yaml"),
                    &ResponseMeta {
                        format: result.format,
                        request_id: result.request_id.clone(),
                        usage: result.usage.clone(),
                    },
                )?;
                job.request_id = result.request_id.or(job.request_id.clone());
                job.usage = result.usage;
                job.state = State::Queued;
                job.error = None;
            }
            Ok(crate::providers::Reconciled::Pending) => {
                job.error=Some("Provider interaction is still pending or requires action; no new request submitted".into());
            }
            Ok(crate::providers::Reconciled::Failed) => {
                job.state = State::Failed;
                job.error=Some("Provider confirmed failed/cancelled interaction; a later resume may explicitly make another bounded attempt".into());
            }
            Err(error) => {
                job.error = Some(format!(
                    "Provider reconciliation did not establish the outcome: {error}"
                ));
            }
        }
    }
    run.state = if run.jobs.iter().any(|j| j.state == State::Unknown) {
        State::Unknown
    } else if run.jobs.iter().any(|j| j.state == State::Queued) {
        State::Queued
    } else if run.jobs.iter().all(|j| j.state == State::Succeeded) {
        State::Succeeded
    } else if run.jobs.iter().any(|j| j.state == State::Succeeded) {
        State::Partial
    } else {
        State::Failed
    };
    save(&path.join("run.yaml"), &run)?;
    Ok(run)
}

/// Stage a user-attested recovered response; resume performs normal publication.
pub fn recover(
    root: &Path,
    run_id: &str,
    job_id: &str,
    file: &Path,
    format: &str,
    fingerprint: &str,
    evidence: &Path,
) -> Result<Run> {
    let path = run_path(root, run_id)?;
    let guard = lock(&path.join("worker.lock"))?;
    if !guard.try_lock_exclusive()? {
        return Err(Error::input("A worker owns this run"));
    }
    let run = load(root, run_id)?;
    let job = run
        .jobs
        .iter()
        .find(|j| j.input.id == job_id)
        .ok_or_else(|| Error::input("Unknown job ID"))?;
    if job.state != State::Unknown || !job.input.turns.is_empty() || job.fingerprint != fingerprint
    {
        return Err(Error::input(
            "Recovery requires an uncertain leaf job and its exact recorded fingerprint",
        ));
    }
    let allowed = match job.input.media {
        Media::Image | Media::Infographic => matches!(format, "png" | "jpeg" | "webp"),
        Media::Speech | Media::Music | Media::Sfx => matches!(format, "wav" | "mp3" | "flac"),
        Media::Text => format == "md",
        _ => false,
    };
    if !allowed {
        return Err(Error::input("Recovered format does not match job media"));
    }
    let bytes = fs::read(file)?;
    integrity(&bytes, format)?;
    let evidence_text = text(
        evidence.parent().unwrap_or(Path::new(".")),
        evidence.file_name().unwrap().as_ref(),
    )?;
    if evidence_text.len() > 16384 {
        return Err(Error::input(
            "Recovery evidence must be at most 16 KiB; omit credentials and account details",
        ));
    }
    let raw = path.join("responses").join(format!("{job_id}.bin"));
    let meta = raw.with_extension("yaml");
    if raw.exists() || meta.exists() {
        return Err(Error::input(
            "A saved response already exists; inspect it before resume",
        ));
    }
    let receipt = serde_json::json!({"provenance":"user_attested","job":job_id,"fingerprint":fingerprint,"request_id":job.request_id,"sha256":hash(&bytes),"format":format,"evidence":evidence_text});
    atomic(&raw, &bytes, false)?;
    save(&raw.with_extension("recovery.yaml"), &receipt)?;
    save(
        &meta,
        &ResponseMeta {
            format: format.into(),
            request_id: job.request_id.clone(),
            usage: job.usage.clone(),
        },
    )?;
    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_preserves_a_worker_completion_between_observation_and_lock() {
        let root = tempfile::tempdir().unwrap();
        let input: Effective = serde_json::from_value(serde_json::json!({
            "id":"fast-failure","media":"image","selection":{"provider":"openai","model":"gpt-image-2.5-flare"},
            "prompt":"Offline fixture","settings":{},"references":[],"depends_on":[]
        })).unwrap();
        let mut run = submit(
            root.path(),
            vec![input],
            Limits::default(),
            Submit::default(),
        )
        .unwrap();
        run.state = State::Running;
        run.jobs[0].state = State::Running;
        let file = run_path(root.path(), &run.id).unwrap().join("run.yaml");
        save(&file, &run).unwrap();
        let observed = load(root.path(), &run.id).unwrap();
        // Simulate the worker's final write/release before the status lock probe.
        run.state = State::Failed;
        run.jobs[0].state = State::Failed;
        run.jobs[0].error = Some("missing_credentials: fixture".into());
        save(&file, &run).unwrap();
        let receipt = fs::read(&file).unwrap();
        let result = status_after_observation(root.path(), &run.id, observed).unwrap();
        assert_eq!(result.state, State::Failed);
        assert_eq!(
            result.jobs[0].error.as_deref(),
            Some("missing_credentials: fixture")
        );
        assert_eq!(fs::read(file).unwrap(), receipt);
    }
    #[test]
    fn lowered_host_ceilings_count_inflight_slots_across_providers() {
        let root = tempfile::tempdir().unwrap();
        let high = crate::setup::HostLimits {
            schema_version: 1,
            concurrency: 8,
            per_provider: 4,
        };
        let mut held = vec![];
        for _ in 0..4 {
            held.push(
                try_slots(root.path(), Provider::Fish, &high)
                    .unwrap()
                    .unwrap(),
            );
        }
        held.push(
            try_slots(root.path(), Provider::Openai, &high)
                .unwrap()
                .unwrap(),
        );
        assert!(
            try_slots(root.path(), Provider::Fish, &high)
                .unwrap()
                .is_none()
        );
        let low = crate::setup::HostLimits {
            schema_version: 1,
            concurrency: 3,
            per_provider: 2,
        };
        assert!(
            try_slots(root.path(), Provider::Google, &low)
                .unwrap()
                .is_none()
        );
        held.pop();
        held.pop();
        assert!(
            try_slots(root.path(), Provider::Google, &low)
                .unwrap()
                .is_none()
        );
        held.pop();
        assert!(
            try_slots(root.path(), Provider::Fish, &low)
                .unwrap()
                .is_none()
        );
        let extra = try_slots(root.path(), Provider::Google, &low)
            .unwrap()
            .unwrap();
        assert!(
            try_slots(root.path(), Provider::Openai, &low)
                .unwrap()
                .is_none()
        );
        drop(extra);
        assert!(
            try_slots(root.path(), Provider::Openai, &low)
                .unwrap()
                .is_some()
        );
    }
}
