use mediaagent::{audio, contracts::*, project, setup, storage, validation};
use std::{fs, path::Path};
fn recording(root: &Path, granted: bool) -> VoiceSample {
    fs::write(root.join("sample.wav"), audio::wav(&[0.1; 2400]).unwrap()).unwrap();
    fs::write(
        root.join("transcript.md"),
        "Exact sample.\nWith two lines!\n",
    )
    .unwrap();
    fs::write(
        root.join("permission.md"),
        "Fixture authorization from the fictional test speaker for these offline tests.",
    )
    .unwrap();
    storage::save(
        &root.join("consent.yaml"),
        &VoiceConsent {
            schema_version: 1,
            subject: "Fixture speaker".into(),
            authorized_by: "Fixture owner".into(),
            granted,
            purposes: vec![ConsentPurpose::ReferenceSynthesis],
            evidence: "permission.md".into(),
        },
    )
    .unwrap();
    VoiceSample {
        audio: root.join("sample.wav"),
        transcript: root.join("transcript.md"),
        consent: root.join("consent.yaml"),
    }
}
#[test]
fn recording_permission_transcript_and_portable_identity_survive_source_removal() {
    let root = tempfile::tempdir().unwrap();
    let samples = tempfile::tempdir().unwrap();
    let sample = recording(samples.path(), true);
    let original =
        validation::voice_sample(Path::new("."), &sample, ConsentPurpose::ReferenceSynthesis)
            .unwrap();
    assert_eq!(
        original.transcript.as_deref(),
        Some("Exact sample.\nWith two lines!\n")
    );
    assert!(
        validation::voice_sample(Path::new("."), &sample, ConsentPurpose::SavedVoiceClone).is_err()
    );
    let path = project::add_persona(root.path(), "narrator").unwrap();
    let mut character: Character = storage::yaml(&path).unwrap();
    character.voice = Some(Voice {
        description: "voice.md".into(),
        selection: Selection {
            provider: Provider::Fish,
            model: "s2.1-pro-free".into(),
        },
        binding: None,
        references: vec![],
        samples: vec![],
    });
    storage::save(&path, &character).unwrap();
    for _ in 0..2 {
        project::accept_sample(&path, &sample).unwrap();
    }
    assert_eq!(
        validation::character(&path)
            .unwrap()
            .voice
            .unwrap()
            .samples
            .len(),
        1
    );
    let bundle = root.path().join("bundle");
    project::bundle(&path, &bundle, None).unwrap();
    drop(samples);
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
    let destination = tempfile::tempdir().unwrap();
    let imported = project::import(destination.path(), &bundle, None).unwrap();
    let c = validation::character(&imported).unwrap();
    let r = validation::voice_sample(
        imported.parent().unwrap(),
        &c.voice.unwrap().samples[0],
        ConsentPurpose::ReferenceSynthesis,
    )
    .unwrap();
    assert_eq!(r.sha256, original.sha256);
    assert_eq!(r.transcript, original.transcript);
    assert_eq!(r.consent_hash, original.consent_hash);
}
#[test]
fn missing_or_denied_permission_is_rejected_before_acceptance() {
    let source = tempfile::tempdir().unwrap();
    let sample = recording(source.path(), false);
    assert!(
        validation::voice_sample(Path::new("."), &sample, ConsentPurpose::ReferenceSynthesis)
            .is_err()
    );
    let mut consent: VoiceConsent = storage::yaml(&sample.consent).unwrap();
    consent.granted = true;
    storage::save(&sample.consent, &consent).unwrap();
    fs::remove_file(source.path().join("permission.md")).unwrap();
    assert!(
        validation::voice_sample(Path::new("."), &sample, ConsentPurpose::ReferenceSynthesis)
            .is_err()
    );
}
#[test]
fn host_limits_are_explicit_validated_and_preserved_on_invalid_updates() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("host.yaml");
    let mut limits = setup::HostLimits {
        schema_version: 1,
        concurrency: 16,
        per_provider: 8,
    };
    setup::save_host(&path, &limits).unwrap();
    let before = fs::read(&path).unwrap();
    limits.per_provider = 17;
    assert!(setup::save_host(&path, &limits).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
    limits.per_provider = 8;
    limits.concurrency = 4;
    assert!(setup::save_host(&path, &limits).is_err());
    let mut config = Config::default();
    config.defaults.insert(
        Media::Music,
        Selection {
            provider: Provider::Fish,
            model: "s2.1-pro".into(),
        },
    );
    assert!(validation::config(&config).is_err());
}

#[test]
fn long_elevenlabs_scripts_split_exactly_and_podcast_needs_no_fictitious_default() {
    let root = tempfile::tempdir().unwrap();
    let text = "Hello, this is a narration sentence. ".repeat(160);
    fs::write(root.path().join("script.md"), &text).unwrap();
    fs::write(root.path().join("speech.yaml"),"schema_version: 1\nid: narration\nmedia: speech\nselection: {provider: elevenlabs, model: eleven_v3}\ntext_file: script.md\nsettings: {voice: fixture-binding}\n").unwrap();
    let job: Job = storage::yaml(&root.path().join("speech.yaml")).unwrap();
    let e = validation::resolve(&job, root.path(), &Config::default()).unwrap();
    assert_eq!(e.turns.len(), 2);
    assert_eq!(
        e.turns
            .iter()
            .map(|t| t.text.as_deref().unwrap())
            .collect::<String>(),
        text
    );
    assert!(
        e.turns
            .iter()
            .all(|t| t.text.as_ref().unwrap().chars().count() <= 5000)
    );
    fs::write(root.path().join("podcast.yaml"),"schema_version: 1\nid: podcast\nmedia: podcast\nturns:\n  - id: host\n    text_file: script.md\n    selection: {provider: elevenlabs, model: eleven_v3}\n    settings: {voice: fixture-binding}\n").unwrap();
    let job: Job = storage::yaml(&root.path().join("podcast.yaml")).unwrap();
    let e = validation::resolve(&job, root.path(), &Config::default()).unwrap();
    assert_eq!(e.turns[0].text.as_ref().unwrap(), &text);
    assert_eq!(e.turns[0].turns.len(), 2);
}
