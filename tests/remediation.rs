use async_trait::async_trait;
use mediaagent::{
    audio, contracts::*, engine, maintenance, planning, project, storage::*, validation,
    verification,
};
use mediaagent::{
    error::{Error, Result},
    providers::{Adapter, Generated},
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
fn png() -> Vec<u8> {
    let mut c = std::io::Cursor::new(vec![]);
    image::RgbImage::from_pixel(4, 4, image::Rgb([1, 2, 3]))
        .write_to(&mut c, image::ImageFormat::Png)
        .unwrap();
    c.into_inner()
}
fn root() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    project::init(t.path(), &[]).unwrap();
    fs::write(t.path().join("brief.md"), "Playing together in a forest.").unwrap();
    t
}
fn job(root: &Path) -> Job {
    let job:Job=serde_json::from_value(json!({"schema_version":2,"id":"scene","media":"image","selection":{"provider":"openai","model":"gpt-image-1"},"brief":"brief.md"})).unwrap();
    save(&root.join("job.yaml"), &job).unwrap();
    job
}
fn character(root: &Path, id: &str) -> PathBuf {
    let p = project::add_persona(root, id).unwrap();
    let mut c: Character = yaml(&p).unwrap();
    let b = p.parent().unwrap();
    fs::write(b.join("ref.png"), png()).unwrap();
    c.visual=Some(serde_json::from_value(json!({"selection":{"provider":"openai","model":"gpt-image-1"},"description":"visual.md","references":["ref.png"]})).unwrap());
    save(&p, &c).unwrap();
    p
}
fn cast(root: &Path) -> Job {
    let mut j = job(root);
    j.cast = vec![
        CastMember {
            alias: "mira".into(),
            character: character(root, "mira"),
            direction: None,
        },
        CastMember {
            alias: "kito".into(),
            character: character(root, "kito"),
            direction: None,
        },
    ];
    j
}
#[test]
fn cast_preserves_order_pins_and_hashes_direction() {
    let r = root();
    let mut j = cast(r.path());
    j.settings.input_fidelity = Some("high".into());
    let cfg = Config::default();
    let a = validation::resolve(&j, r.path(), &cfg).unwrap();
    assert_eq!(a.references.len(), 2);
    assert!(a.references[0].name.starts_with("mira"));
    assert!(a.references[1].name.starts_with("kito"));
    assert_eq!(a.settings.input_fidelity.as_deref(), Some("high"));
    fs::write(r.path().join("direction.md"), "Mira is on the left.").unwrap();
    j.cast[0].direction = Some("direction.md".into());
    let b = validation::resolve(&j, r.path(), &cfg).unwrap();
    assert_ne!(
        validation::fingerprint(&a).unwrap(),
        validation::fingerprint(&b).unwrap()
    );
}
#[test]
fn cast_rejects_alias_conflicts_missing_references_and_incompatible_models() {
    let r = root();
    let mut j = cast(r.path());
    j.cast[1].alias = "mira".into();
    assert!(validation::resolve(&j, r.path(), &Config::default()).is_err());
    j.cast[1].alias = "kito".into();
    let p = &j.cast[1].character;
    let mut c: Character = yaml(p).unwrap();
    c.visual.as_mut().unwrap().selection.model = "gpt-image-1.5".into();
    save(p, &c).unwrap();
    assert!(validation::resolve(&j, r.path(), &Config::default()).is_err());
    c.visual.as_mut().unwrap().selection.model = "gpt-image-1".into();
    c.visual.as_mut().unwrap().references.clear();
    save(p, &c).unwrap();
    assert!(validation::resolve(&j, r.path(), &Config::default()).is_err());
}
#[test]
fn cast_and_assembly_require_v2_and_reject_singular_character() {
    let r = root();
    let mut j = cast(r.path());
    j.schema_version = 1;
    assert!(validation::resolve(&j, r.path(), &Config::default()).is_err());
    j.schema_version = 2;
    j.character = Some(j.cast[0].character.clone());
    assert!(validation::resolve(&j, r.path(), &Config::default()).is_err());
}
#[test]
fn planner_selected_character_bundle_survives_original_removal() {
    let r = root();
    let p = character(r.path(), "mira");
    let stage = tempfile::tempdir().unwrap();
    let v =
        planning::prepare_inputs(r.path(), stage.path(), std::slice::from_ref(&p), &[]).unwrap();
    assert_eq!(
        Path::new(v[0]["manifest"].as_str().unwrap()),
        Path::new("_inputs/characters/mira/character.yaml")
    );
    fs::remove_dir_all(p.parent().unwrap()).unwrap();
    let data=json!({"files":[{"path":"job.yaml","content":"schema_version: 2\nid: scene\nmedia: image\ncharacter: _inputs/characters/mira/character.yaml\nbrief: scene.md\n"},{"path":"scene.md","content":"Walk through a forest."}]}).to_string();
    let path = planning::stage(&data, "job", stage.path(), &Config::default()).unwrap();
    let j: Job = yaml(&path).unwrap();
    assert_eq!(
        validation::resolve(&j, stage.path(), &Config::default())
            .unwrap()
            .references
            .len(),
        1
    );
}
#[test]
fn planner_cannot_overwrite_reserved_assets() {
    let r = root();
    let data =
        json!({"files":[{"path":"_inputs/character.yaml","content":"replacement"}]}).to_string();
    assert!(planning::stage(&data, "job", r.path(), &Config::default()).is_err());
}
#[test]
fn blobs_roundtrip_deduplicate_and_detect_corruption() {
    let r = root();
    let j = cast(r.path());
    let input = validation::resolve(&j, r.path(), &Config::default()).unwrap();
    let run = engine::submit(
        r.path(),
        vec![input.clone()],
        Limits::default(),
        Default::default(),
    )
    .unwrap();
    let p = engine::run_path(r.path(), &run.id)
        .unwrap()
        .join("run.yaml");
    let raw = fs::read_to_string(&p).unwrap();
    assert!(!raw.contains("base64:"));
    assert!(raw.contains("blob:"));
    let loaded = engine::load(r.path(), &run.id).unwrap();
    assert_eq!(
        validation::fingerprint(&input).unwrap(),
        validation::fingerprint(&loaded.jobs[0].input).unwrap()
    );
    assert_eq!(
        fs::read_dir(r.path().join("mediaagent/blobs"))
            .unwrap()
            .count(),
        1
    );
    fs::write(
        r.path()
            .join("mediaagent/blobs")
            .join(&input.references[0].sha256),
        b"bad",
    )
    .unwrap();
    assert!(engine::load(r.path(), &run.id).is_err());
}
#[test]
fn metadata_rebuild_does_not_need_reference_bytes() {
    let r = root();
    let j = cast(r.path());
    let input = validation::resolve(&j, r.path(), &Config::default()).unwrap();
    let run = engine::submit(r.path(), vec![input], Limits::default(), Default::default()).unwrap();
    let p = engine::run_path(r.path(), &run.id)
        .unwrap()
        .join("run.yaml");
    let before = mediaagent::blobs::read_metadata(&p).unwrap();
    fs::remove_file(p.with_file_name("metadata.json")).unwrap();
    assert_eq!(before, mediaagent::blobs::read_metadata(&p).unwrap());
}
#[test]
fn migration_preview_is_nonmutating_and_backup_is_exact() {
    let r = root();
    let mut j = job(r.path());
    j.schema_version = 1;
    let p = r.path().join("job.yaml");
    save(&p, &j).unwrap();
    let original = fs::read(&p).unwrap();
    maintenance::migrate(r.path(), &p, "job", false).unwrap();
    assert_eq!(fs::read(&p).unwrap(), original);
    maintenance::migrate(r.path(), &p, "job", true).unwrap();
    assert_eq!(fs::read(p.with_extension("v1.bak")).unwrap(), original);
    assert_eq!(yaml::<Job>(&p).unwrap().schema_version, 2);
    maintenance::migrate(r.path(), &p, "job", true).unwrap();
    assert_eq!(fs::read(p.with_extension("v1.bak")).unwrap(), original);
}
#[test]
fn cleanup_removes_only_orphans_and_rejects_stale_plan() {
    let r = root();
    let bytes = png();
    let sha = hash(&bytes);
    let blob = r.path().join("mediaagent/blobs").join(&sha);
    atomic(&blob, &bytes, false).unwrap();
    let p = r.path().join("cleanup.yaml");
    let plan = maintenance::storage_plan(r.path()).unwrap();
    save(&p, &plan).unwrap();
    fs::write(r.path().join("mediaagent/new.md"), "Changed").unwrap();
    assert!(maintenance::clean(r.path(), &p).is_err());
    assert!(blob.exists());
    save(&p, &maintenance::storage_plan(r.path()).unwrap()).unwrap();
    maintenance::clean(r.path(), &p).unwrap();
    assert!(!blob.exists());
    assert!(r.path().join("mediaagent/new.md").exists());
}
#[test]
fn cleanup_protects_references_in_every_run() {
    let r = root();
    let j = cast(r.path());
    let input = validation::resolve(&j, r.path(), &Config::default()).unwrap();
    engine::submit(r.path(), vec![input], Limits::default(), Default::default()).unwrap();
    let p = maintenance::storage_plan(r.path()).unwrap();
    assert_eq!(p["candidates"].as_array().unwrap().len(), 0);
}
#[test]
fn assembly_applies_pauses_gain_and_peak_ceiling_without_changing_sources() {
    let clips = vec![
        ("a".into(), vec![0.1; 24000]),
        ("b".into(), vec![0.2; 24000]),
    ];
    assert_eq!(audio::assemble(&clips, None).unwrap().len(), 48000);
    let a = Assembly {
        pause_ms: 100,
        turn_gain_db: std::collections::BTreeMap::from([("b".into(), 6.0)]),
        loudness_lufs: None,
    };
    let out = audio::assemble(&clips, Some(&a)).unwrap();
    assert_eq!(out.len(), 50400);
    assert!(out[24000..26400].iter().all(|x| *x == 0.0));
    assert!((out[26400] - 0.39905).abs() < 0.001);
    assert_eq!(clips[1].1[0], 0.2);
    let invalid = Assembly {
        pause_ms: 10001,
        ..Default::default()
    };
    assert!(audio::assemble(&clips, Some(&invalid)).is_err());
}
#[test]
fn loudness_normalization_reaches_target_when_peak_allows() {
    let samples = (0..72000)
        .map(|i| ((i as f32 / 24000.0) * 440.0 * std::f32::consts::TAU).sin() * 0.03)
        .collect::<Vec<_>>();
    let out = audio::assemble(
        &[("tone".into(), samples)],
        Some(&Assembly {
            loudness_lufs: Some(-20.0),
            ..Default::default()
        }),
    )
    .unwrap();
    let mut m = ebur128::EbuR128::new(1, 24000, ebur128::Mode::I).unwrap();
    m.add_frames_f32(&out).unwrap();
    assert!((m.loudness_global().unwrap() + 20.0).abs() < 0.1);
    assert!(
        out.iter()
            .all(|s| s.abs() <= 10_f32.powf(-1.0 / 20.0) + 1e-6)
    );
}
fn suite(r: &Path) -> PathBuf {
    job(r);
    let s = verification::Suite {
        schema_version: 1,
        id: "check".into(),
        description: "Single image".into(),
        jobs: vec!["job.yaml".into()],
        limits: Limits {
            requests: 1,
            images: 1,
            attempts: 1,
            ..Default::default()
        },
        criteria: vec!["Human identity acceptance".into()],
    };
    let p = r.join("suite.yaml");
    save(&p, &s).unwrap();
    p
}
#[test]
fn verification_preview_is_offline_bounded_and_content_bound() {
    let r = root();
    let p = suite(r.path());
    let a = verification::prepare(r.path(), &p).unwrap();
    assert_eq!(a.preview["estimated_reservation"]["requests"], 1);
    assert_eq!(
        fs::read_dir(r.path().join("mediaagent/runs"))
            .unwrap()
            .count(),
        0
    );
    fs::write(r.path().join("brief.md"), "Changed scene").unwrap();
    let b = verification::prepare(r.path(), &p).unwrap();
    assert_ne!(a.hash, b.hash);
    assert!(verification::submit(r.path(), b, &a.hash).is_err());
    let mut s: verification::Suite = yaml(&p).unwrap();
    s.limits.images = 0;
    save(&p, &s).unwrap();
    assert!(verification::prepare(r.path(), &p).is_err());
}
struct Uncertain {
    calls: AtomicUsize,
}
#[async_trait]
impl Adapter for Uncertain {
    async fn generate(&self, _: &Effective) -> Result<Generated> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(Error::unknown("fixture interruption"))
    }
}
#[tokio::test]
async fn manual_recovery_preserves_attempts_and_never_resubmits() {
    let r = root();
    let input = validation::resolve(&job(r.path()), r.path(), &Config::default()).unwrap();
    let run = engine::submit(r.path(), vec![input], Limits::default(), Default::default()).unwrap();
    let adapter = Arc::new(Uncertain {
        calls: AtomicUsize::new(0),
    });
    let run = engine::work(r.path(), &run.id, adapter.clone(), false)
        .await
        .unwrap();
    assert_eq!(run.jobs[0].state, State::Unknown);
    let file = r.path().join("recovered.png");
    fs::write(&file, png()).unwrap();
    let evidence = r.path().join("evidence.md");
    fs::write(
        &evidence,
        "Matched original request and downloaded its result; fixture.",
    )
    .unwrap();
    assert!(engine::recover(r.path(), &run.id, "scene", &file, "png", "wrong", &evidence).is_err());
    engine::recover(
        r.path(),
        &run.id,
        "scene",
        &file,
        "png",
        &run.jobs[0].fingerprint,
        &evidence,
    )
    .unwrap();
    let done = engine::work(r.path(), &run.id, adapter.clone(), false)
        .await
        .unwrap();
    assert_eq!(done.state, State::Succeeded);
    assert_eq!(adapter.calls.load(Ordering::SeqCst), 1);
    assert_eq!(done.jobs[0].attempts, 1);
    let report = r.path().join("comparison.md");
    maintenance::comparison(r.path(), &[done.jobs[0].artifacts[0].id.clone()], &report).unwrap();
    assert!(report.exists());
    assert_eq!(done.jobs[0].artifacts[0].quality, Quality::Unchecked);
}

#[test]
fn planner_rejects_relabelled_original_asset_models() {
    let r = root();
    let image = r.path().join("original.png");
    fs::write(&image, png()).unwrap();
    let stage = tempfile::tempdir().unwrap();
    planning::prepare_inputs(r.path(), stage.path(), &[], std::slice::from_ref(&image)).unwrap();
    // Provide the same hash-bound receipt that preparation obtains from real run metadata.
    let inventory_path = stage.path().join("_inputs/inventory.yaml");
    let mut inventory: serde_json::Value = yaml(&inventory_path).unwrap();
    inventory[0]["origin"] = json!({"selection":{"provider":"openai","model":"gpt-image-1"},"settings":{},"locale":null,"sha256":hash(&png()),"artifact":null});
    save(&inventory_path, &inventory).unwrap();
    let job = format!(
        "schema_version: 2\nid: derived\nmedia: image\nselection: {{provider: openai, model: gpt-image-1.5}}\nbrief: brief.md\nreferences:\n  - path: _inputs/assets/0-original.png\n    role: identity\n    origin:\n      selection: {{provider: openai, model: gpt-image-1.5}}\n      settings: {{}}\n      sha256: {}\n",
        hash(&png())
    );
    let response=json!({"files":[{"path":"job.yaml","content":job},{"path":"brief.md","content":"Preserve the selected original in a forest."}]}).to_string();
    assert!(planning::stage(&response, "job", stage.path(), &Config::default()).is_err());
}
#[test]
fn verification_execution_with_missing_keys_is_skipped_not_passed() {
    let r = root();
    let p = suite(r.path());
    let preview = verification::prepare(r.path(), &p).unwrap();
    let mut command = assert_cmd::Command::new(assert_cmd::cargo::cargo_bin!("mediaagent"));
    for provider in [
        Provider::Openai,
        Provider::Google,
        Provider::Fish,
        Provider::Openrouter,
        Provider::Elevenlabs,
    ] {
        command.env_remove(provider.key());
    }
    let result = command
        .args([
            "--project",
            r.path().to_str().unwrap(),
            "--json",
            "verify",
            "--suite",
            p.to_str().unwrap(),
            "--execute",
            "--preview-hash",
            &preview.hash,
        ])
        .assert()
        .code(3);
    let value: serde_json::Value = serde_json::from_slice(&result.get_output().stdout).unwrap();
    assert_eq!(value["data"]["skipped"][0]["status"], "skipped");
    assert_eq!(
        fs::read_dir(r.path().join("mediaagent/runs"))
            .unwrap()
            .count(),
        0
    );
}
#[test]
fn usage_keeps_voice_operations_and_unknown_costs_separate() {
    let r = root();
    let dir = r.path().join("mediaagent/runs/voice-test");
    fs::create_dir_all(&dir).unwrap();
    let op = Operation {
        schema_version: 1,
        id: "operation".into(),
        selection: Selection {
            provider: Provider::Elevenlabs,
            model: "eleven_ttv_v3".into(),
        },
        input: json!({}),
        state: State::Succeeded,
        attempts: 1,
        request_id: Some("fixture".into()),
        usage: None,
        error: None,
        retry_not_before: None,
        result: None,
        history: vec![],
    };
    save(&dir.join("design-operation.yaml"), &op).unwrap();
    save(
        &dir.join("budget.yaml"),
        &Budget {
            requests: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let report = maintenance::usage(r.path()).unwrap();
    assert_eq!(report["operations"].as_array().unwrap().len(), 1);
    assert_eq!(report["budget_groups"].as_array().unwrap().len(), 1);
    assert!(report["currency_total"].is_null());
}
#[cfg(unix)]
#[test]
fn cleanup_refuses_symlinks_and_does_not_touch_their_targets() {
    let r = root();
    let external = tempfile::tempdir().unwrap();
    fs::write(external.path().join("keep"), "keep").unwrap();
    std::os::unix::fs::symlink(external.path(), r.path().join("mediaagent/linked")).unwrap();
    assert!(maintenance::storage_plan(r.path()).is_err());
    assert!(external.path().join("keep").exists());
}
