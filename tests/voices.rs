use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use mediaagent::{
    contracts::*,
    error::Result,
    operations::Output,
    project, storage,
    voices::{self, Adapter, CreateOptions, Preview},
};
use serde_json::json;
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fake {
    designs: AtomicUsize,
    creates: AtomicUsize,
    verification: Option<bool>,
    checks: AtomicUsize,
}
impl Fake {
    fn new(verification: Option<bool>) -> Self {
        Self {
            designs: AtomicUsize::new(0),
            creates: AtomicUsize::new(0),
            verification,
            checks: AtomicUsize::new(0),
        }
    }
}
#[async_trait]
impl Adapter for Fake {
    async fn clone_voice(&self, name: &str, _: &str, recordings: &[Reference]) -> Result<Output> {
        assert_eq!(name, "Offline narrator");
        assert_eq!(recordings.len(), 1);
        assert!(recordings[0].consent_hash.is_some());
        self.creates.fetch_add(1, Ordering::SeqCst);
        Ok(Output {
            value: json!({"voice_id":"saved-clone-1","requires_verification":self.verification}),
            request_id: Some("clone-receipt".into()),
            usage: None,
        })
    }
    async fn verify(&self, voice: &str) -> Result<Output> {
        self.checks.fetch_add(1, Ordering::SeqCst);
        Ok(Output {
            value: json!({"voice_id":voice,"requires_verification":true,"is_verified":true}),
            request_id: Some("verification-id".into()),
            usage: None,
        })
    }
    async fn design(&self, _: &Selection, _: &str, _: &str) -> Result<Output> {
        self.designs.fetch_add(1, Ordering::SeqCst);
        let bytes = mediaagent::audio::wav(&[0.1; 2400])?;
        let previews = vec![Preview {
            id: "candidate-1".into(),
            audio: Reference {
                origin: None,
                transcript: None,
                consent_hash: None,
                role: ReferenceRole::Identity,
                name: "sample.wav".into(),
                mime: "audio/wav".into(),
                sha256: storage::hash(&bytes),
                base64: STANDARD.encode(bytes),
            },
            language: Some("en".into()),
        }];
        Ok(Output {
            value: serde_json::to_value(previews)?,
            request_id: Some("design-id".into()),
            usage: None,
        })
    }
    async fn create(&self, _: &str, _: &str, candidate: &str) -> Result<Output> {
        assert_eq!(candidate, "candidate-1");
        self.creates.fetch_add(1, Ordering::SeqCst);
        Ok(Output {
            value: json!({"voice_id":"saved-voice-1","requires_verification":self.verification}),
            request_id: Some("create-id".into()),
            usage: None,
        })
    }
}
fn character(root: &std::path::Path) -> std::path::PathBuf {
    let path = project::add_persona(root, "narrator").unwrap();
    fs::write(path.parent().unwrap().join("voice.md"),"A warm fictional narrator with a steady mid-range pitch, clear diction, and a relaxed conversational pace.").unwrap();
    let mut c: Character = storage::yaml(&path).unwrap();
    c.voice = Some(Voice {
        samples: vec![],
        description: "voice.md".into(),
        selection: Selection {
            provider: Provider::Elevenlabs,
            model: "eleven_flash_v2_5".into(),
        },
        binding: None,
        references: vec![],
    });
    storage::save(&path, &c).unwrap();
    path
}
fn draft(root: &std::path::Path, path: &std::path::Path) -> VoiceDesign {
    voices::draft(
        root,
        path,
        Selection {
            provider: Provider::Elevenlabs,
            model: "eleven_ttv_v3".into(),
        },
        &"Hello, this exact script is for a fictional narrator preview. ".repeat(3),
        Some("test-design"),
        Limits::default(),
    )
    .unwrap()
}
#[tokio::test]
async fn design_preview_creation_and_binding_are_separate_and_idempotent() {
    let root = tempfile::tempdir().unwrap();
    let path = character(root.path());
    let fake = Fake::new(Some(false));
    let created = voices::design(root.path(), draft(root.path(), &path), false, &fake)
        .await
        .unwrap();
    assert_eq!(fake.designs.load(Ordering::SeqCst), 1);
    assert_eq!(fake.creates.load(Ordering::SeqCst), 0);
    assert_eq!(created.candidates[0].artifact.quality, Quality::Unchecked);
    assert!(
        storage::yaml::<Character>(&path)
            .unwrap()
            .voice
            .unwrap()
            .binding
            .is_none()
    );
    let recovered = voices::design(root.path(), draft(root.path(), &path), false, &fake)
        .await
        .unwrap();
    assert_eq!(
        recovered.candidates[0].artifact.id,
        created.candidates[0].artifact.id
    );
    assert_eq!(fake.designs.load(Ordering::SeqCst), 1);
    let result = voices::create(
        root.path(),
        &path,
        &created,
        "candidate-1",
        "Narrator",
        CreateOptions::default(),
        &fake,
    )
    .await
    .unwrap();
    assert_eq!(result["bound"], false);
    assert!(
        storage::yaml::<Character>(&path)
            .unwrap()
            .voice
            .unwrap()
            .binding
            .is_none()
    );
    for _ in 0..2 {
        let result = voices::create(
            root.path(),
            &path,
            &created,
            "candidate-1",
            "Narrator",
            CreateOptions {
                bind: true,
                ..Default::default()
            },
            &fake,
        )
        .await
        .unwrap();
        assert_eq!(result["bound"], true);
    }
    assert_eq!(fake.creates.load(Ordering::SeqCst), 1);
    let c: Character = storage::yaml(&path).unwrap();
    assert_eq!(
        c.voice.as_ref().unwrap().binding.as_deref(),
        Some("saved-voice-1")
    );
    assert!(c.voice.unwrap().references.is_empty());
}
#[tokio::test]
async fn provider_verification_and_changed_identity_prevent_binding() {
    for status in [Some(true), None] {
        let root = tempfile::tempdir().unwrap();
        let path = character(root.path());
        let fake = Fake::new(status);
        let created = voices::design(root.path(), draft(root.path(), &path), false, &fake)
            .await
            .unwrap();
        let result = voices::create(
            root.path(),
            &path,
            &created,
            "candidate-1",
            "Narrator",
            CreateOptions {
                bind: true,
                ..Default::default()
            },
            &fake,
        )
        .await
        .unwrap();
        assert_eq!(result["bound"], false);
        assert!(
            storage::yaml::<Character>(&path)
                .unwrap()
                .voice
                .unwrap()
                .binding
                .is_none()
        );
        fs::write(
            path.parent().unwrap().join("persona.md"),
            "Changed identity",
        )
        .unwrap();
        assert!(
            voices::create(
                root.path(),
                &path,
                &created,
                "candidate-1",
                "Narrator",
                CreateOptions::default(),
                &fake
            )
            .await
            .is_err()
        );
        assert_eq!(fake.creates.load(Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn candidate_hash_budget_and_existing_binding_are_checked_before_creation() {
    let root = tempfile::tempdir().unwrap();
    let path = character(root.path());
    let fake = Fake::new(Some(false));
    let mut initial = draft(root.path(), &path);
    initial.limits.requests = 1;
    let created = voices::design(root.path(), initial, false, &fake)
        .await
        .unwrap();
    let error = voices::create(
        root.path(),
        &path,
        &created,
        "candidate-1",
        "Narrator",
        CreateOptions::default(),
        &fake,
    )
    .await
    .unwrap_err();
    assert_eq!(error.exit, 6);
    assert_eq!(fake.creates.load(Ordering::SeqCst), 0);
    let mut c: Character = storage::yaml(&path).unwrap();
    c.voice.as_mut().unwrap().binding = Some("prior-voice".into());
    storage::save(&path, &c).unwrap();
    assert!(
        voices::create(
            root.path(),
            &path,
            &created,
            "candidate-1",
            "Narrator",
            CreateOptions {
                bind: true,
                ..Default::default()
            },
            &fake
        )
        .await
        .is_err()
    );
    fs::write(&created.candidates[0].artifact.path, b"corrupt").unwrap();
    let error = voices::create(
        root.path(),
        &path,
        &created,
        "candidate-1",
        "Narrator",
        CreateOptions::default(),
        &fake,
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "artifact_corrupt");
    assert_eq!(fake.creates.load(Ordering::SeqCst), 0);
}
#[test]
fn design_requires_exact_script_model_profile_and_bounded_volume() {
    let root = tempfile::tempdir().unwrap();
    let path = character(root.path());
    let mut proposed = draft(root.path(), &path);
    proposed.selection.provider = Provider::Openai;
    assert!(
        voices::validate_design(&proposed.selection, &proposed.description, &proposed.text)
            .is_err()
    );
    proposed.selection.provider = Provider::Elevenlabs;
    assert!(voices::validate_design(&proposed.selection, &proposed.description, "Short").is_err());
    let limits = Limits {
        speech_chars: 100,
        ..Default::default()
    };
    assert_eq!(
        voices::draft(
            root.path(),
            &path,
            proposed.selection,
            &proposed.text,
            None,
            limits
        )
        .unwrap_err()
        .exit,
        6
    );
    assert!(!root.path().join("mediaagent/runs").exists());
}

#[tokio::test]
async fn verification_refresh_binds_the_saved_voice_without_recreating_it() {
    let root = tempfile::tempdir().unwrap();
    let path = character(root.path());
    let fake = Fake::new(Some(true));
    let created = voices::design(root.path(), draft(root.path(), &path), false, &fake)
        .await
        .unwrap();
    let first = voices::create(
        root.path(),
        &path,
        &created,
        "candidate-1",
        "Narrator",
        CreateOptions {
            bind: true,
            ..Default::default()
        },
        &fake,
    )
    .await
    .unwrap();
    assert_eq!(first["bound"], false);
    let refreshed = voices::create(
        root.path(),
        &path,
        &created,
        "candidate-1",
        "Narrator",
        CreateOptions {
            bind: true,
            refresh_verification: true,
            ..Default::default()
        },
        &fake,
    )
    .await
    .unwrap();
    assert_eq!(refreshed["bound"], true);
    let repeated = voices::create(
        root.path(),
        &path,
        &created,
        "candidate-1",
        "Narrator",
        CreateOptions {
            bind: true,
            ..Default::default()
        },
        &fake,
    )
    .await
    .unwrap();
    assert_eq!(repeated["bound"], true);
    assert_eq!(fake.creates.load(Ordering::SeqCst), 1);
    assert_eq!(fake.checks.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn clone_consent_idempotency_and_verification_preserve_canonical_samples() {
    let root = tempfile::tempdir().unwrap();
    let path = character(root.path());
    fs::write(
        root.path().join("sample.wav"),
        mediaagent::audio::wav(&[0.1; 2400]).unwrap(),
    )
    .unwrap();
    fs::write(
        root.path().join("permission.md"),
        "Fixture permission for offline clone tests only.",
    )
    .unwrap();
    let consent_path = root.path().join("consent.yaml");
    let mut consent = VoiceConsent {
        schema_version: 1,
        subject: "Fixture".into(),
        authorized_by: "Fixture owner".into(),
        granted: false,
        purposes: vec![ConsentPurpose::SavedVoiceClone],
        evidence: "permission.md".into(),
    };
    storage::save(&consent_path, &consent).unwrap();
    let request = VoiceClone {
        schema_version: 1,
        id: "clone-test".into(),
        character: path.clone(),
        name: "Offline narrator".into(),
        recordings: vec!["sample.wav".into()],
        consent: "consent.yaml".into(),
        bind: true,
        replace_binding: false,
    };
    let manifest = root.path().join("clone.yaml");
    storage::save(&manifest, &request).unwrap();
    let fake = Fake::new(Some(true));
    assert!(
        voices::clone_voice(
            root.path(),
            &manifest,
            Limits::default(),
            false,
            false,
            false,
            &fake
        )
        .await
        .is_err()
    );
    assert_eq!(fake.creates.load(Ordering::SeqCst), 0);
    consent.granted = true;
    storage::save(&consent_path, &consent).unwrap();
    let dry = voices::clone_voice(
        root.path(),
        &manifest,
        Limits::default(),
        true,
        false,
        false,
        &fake,
    )
    .await
    .unwrap();
    assert_eq!(dry["provider_voice_created"], false);
    assert!(!root.path().join("mediaagent/runs/clone-test").exists());
    let result = voices::clone_voice(
        root.path(),
        &manifest,
        Limits::default(),
        false,
        false,
        false,
        &fake,
    )
    .await
    .unwrap();
    assert_eq!(result["bound"], false);
    assert_eq!(fake.creates.load(Ordering::SeqCst), 1);
    for _ in 0..2 {
        let result = voices::clone_voice(
            root.path(),
            &manifest,
            Limits::default(),
            false,
            false,
            true,
            &fake,
        )
        .await
        .unwrap();
        assert_eq!(result["bound"], true);
    }
    assert_eq!(fake.creates.load(Ordering::SeqCst), 1);
    let c: Character = storage::yaml(&path).unwrap();
    let voice = c.voice.unwrap();
    assert_eq!(voice.binding.as_deref(), Some("saved-clone-1"));
    assert!(voice.references.is_empty());
    assert!(voice.samples.is_empty());
    let status = voices::clone_status(root.path(), "clone-test").unwrap();
    assert_eq!(status.attempts, 1);
    assert_eq!(status.request_id.as_deref(), Some("clone-receipt"));
    fs::write(
        path.parent().unwrap().join("persona.md"),
        "Changed identity",
    )
    .unwrap();
    assert!(
        voices::clone_voice(
            root.path(),
            &manifest,
            Limits::default(),
            false,
            false,
            false,
            &fake
        )
        .await
        .is_err()
    );
    assert_eq!(fake.creates.load(Ordering::SeqCst), 1);
}
