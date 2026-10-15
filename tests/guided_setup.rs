use assert_cmd::cargo::cargo_bin_cmd;
use mediaagent::{catalog, contracts::*, credentials, engine, operations, project, setup, storage};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[test]
fn recommended_init_and_status_are_noninteractive_offline_and_resumable() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("project with spaces");
    let result = cargo_bin_cmd!("mediaagent")
        .args(["--json", "--non-interactive", "init"])
        .arg(&root)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(
        v["data"]["configuration"]["defaults"]["image"]["model"],
        "gpt-image-2.5-flare"
    );
    assert_eq!(
        v["data"]["configuration"]["defaults"]["speech"]["provider"],
        "fish"
    );
    assert_eq!(v["data"]["setup_complete"], false);
    assert!(
        v["data"]["pending_steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("voice"))
    );
    assert!(fs::read(root.join(".env")).unwrap().is_empty());
    let before = fs::read(root.join("mediaagent/config.yaml")).unwrap();
    cargo_bin_cmd!("mediaagent")
        .args(["--non-interactive", "init"])
        .arg(&root)
        .args(["--preset", "empty"])
        .assert()
        .success();
    assert_eq!(
        before,
        fs::read(root.join("mediaagent/config.yaml")).unwrap()
    );
    let shown = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(&root)
        .args(["config", "status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        serde_json::from_slice::<Value>(&shown).unwrap()["data"]["setup_complete"],
        false
    );
}
#[test]
fn readiness_requires_voice_and_keys_and_preserves_explicitly_disabled_features() {
    let t = tempfile::tempdir().unwrap();
    let mut s = setup::Session::new(setup::recommended(), vec!["codex".into(), "claude".into()]);
    assert!(!s.finishable(t.path()));
    fs::write(
        t.path().join(".env"),
        "OPENAI_API_KEY=fixture-openai\nFISH_API_KEY=fixture-fish\n",
    )
    .unwrap();
    assert!(!s.finishable(t.path()));
    for media in [Media::Speech, Media::Podcast] {
        s.config.default_settings.get_mut(&media).unwrap().voice = Some("fixture-voice".into());
    }
    assert!(s.finishable(t.path()));
    let saved = s.save(t.path()).unwrap();
    assert_eq!(saved["setup_complete"], true);
    assert_eq!(saved["account_access_verified"], false);
    assert!(!s.config.defaults.contains_key(&Media::Music));
    assert!(t.path().join(".agents/skills/mediaagent/SKILL.md").exists());
    assert!(t.path().join(".claude/skills/mediaagent/SKILL.md").exists());
}
#[test]
fn credentials_round_trip_literal_characters_without_touching_existing_content() {
    let t = tempfile::tempdir().unwrap();
    let prefix = "# keep this comment\r\nOTHER=keep-me\r\nOPENAI_API_KEY=\r\n";
    fs::write(t.path().join(".env"), prefix).unwrap();
    let value = "fixture $not_expanded `literal` # slash\\ quote\" and ' spaces";
    credentials::save_missing(
        t.path(),
        &BTreeMap::from([(Provider::Openai, value.into())]),
    )
    .unwrap();
    assert_eq!(
        credentials::resolve_with("OPENAI_API_KEY", t.path(), |_| None).unwrap(),
        value
    );
    let after = fs::read_to_string(t.path().join(".env")).unwrap();
    assert!(after.starts_with("# keep this comment\r\nOTHER=keep-me\r\n"));
    credentials::save_missing(
        t.path(),
        &BTreeMap::from([(Provider::Openai, "replacement-not-authorized".into())]),
    )
    .unwrap();
    assert_eq!(after, fs::read_to_string(t.path().join(".env")).unwrap());
    let invalid = BTreeMap::from([(Provider::Fish, "invalid\ncredential".into())]);
    assert!(credentials::save_missing(t.path(), &invalid).is_err());
    assert_eq!(after, fs::read_to_string(t.path().join(".env")).unwrap());
}
#[test]
fn invalid_env_and_duplicate_keys_are_preserved() {
    let t = tempfile::tempdir().unwrap();
    for data in [
        "FISH_API_KEY=\"unclosed\n",
        "FISH_API_KEY=\nFISH_API_KEY=\n",
    ] {
        fs::write(t.path().join(".env"), data).unwrap();
        let error = credentials::save_missing(
            t.path(),
            &BTreeMap::from([(Provider::Fish, "fixture-secret".into())]),
        )
        .unwrap_err();
        assert!(!error.to_string().contains("fixture-secret"));
        assert!(!error.to_string().contains("unclosed"));
        assert_eq!(fs::read_to_string(t.path().join(".env")).unwrap(), data);
    }
}
#[test]
fn catalog_filters_and_current_policy_cannot_be_bypassed_by_arbitrary_text_ids() {
    let t = tempfile::tempdir().unwrap();
    for row in catalog::models() {
        let selection = Selection {
            provider: row.provider,
            model: row.model.clone(),
        };
        assert_eq!(
            catalog::require_current(&selection).is_ok(),
            row.current && row.lifecycle == "stable"
        );
    }
    let filtered = catalog::listing(
        t.path(),
        &catalog::Filter {
            providers: &[Provider::Openai],
            media: Some(Media::Infographic),
            recommended: true,
            ..Default::default()
        },
    );
    let rows = filtered["models"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["model"], "gpt-image-2.5-flare");
    assert!(rows[0]["supported_settings"].as_array().is_some());
    assert!(
        catalog::require_current(&Selection {
            provider: Provider::Openai,
            model: "some-future-model".into()
        })
        .is_err()
    );
    assert!(
        catalog::require_current(&Selection {
            provider: Provider::Google,
            model: "lyria-3-clip-preview".into()
        })
        .is_err()
    );
}
struct Count(AtomicUsize);
#[async_trait::async_trait]
impl mediaagent::providers::Adapter for Count {
    async fn generate(
        &self,
        _: &Effective,
    ) -> mediaagent::error::Result<mediaagent::providers::Generated> {
        self.0.fetch_add(1, Ordering::SeqCst);
        panic!("retired models must never reach the provider")
    }
}
#[tokio::test]
async fn older_projects_are_readable_but_new_submissions_and_side_operations_are_blocked() {
    let t = tempfile::tempdir().unwrap();
    let mut c = Config::default();
    c.defaults.insert(
        Media::Image,
        Selection {
            provider: Provider::Openai,
            model: "gpt-image-1".into(),
        },
    );
    project::init_with_config(t.path(), &[], Some(&c)).unwrap();
    assert_eq!(
        project::config(t.path()).unwrap().defaults[&Media::Image].model,
        "gpt-image-1"
    );
    assert_eq!(setup::status(t.path()).unwrap()["setup_complete"], false);
    let input:Effective=serde_json::from_value(json!({"id":"old","media":"image","selection":{"provider":"openai","model":"gpt-image-1"},"prompt":"Old request","settings":{},"references":[],"depends_on":[],"turns":[]})).unwrap();
    let run = engine::submit(
        t.path(),
        vec![input],
        Limits::default(),
        engine::Submit::default(),
    )
    .unwrap();
    let fake = Arc::new(Count(AtomicUsize::new(0)));
    let done = engine::work(t.path(), &run.id, fake.clone(), false)
        .await
        .unwrap();
    assert_eq!(done.jobs[0].state, State::Blocked);
    assert_eq!(done.jobs[0].attempts, 0);
    assert_eq!(fake.0.load(Ordering::SeqCst), 0);
    let calls = AtomicUsize::new(0);
    let result = operations::perform(
        &t.path().join("old-review.yaml"),
        operations::Request {
            selection: Selection {
                provider: Provider::Openai,
                model: "gpt-4.1-mini".into(),
            },
            input: json!({}),
            attempts: 1,
            retry_unknown: true,
        },
        || panic!("no budget reservation"),
        || async {
            calls.fetch_add(1, Ordering::SeqCst);
            panic!("no provider call")
        },
    )
    .await;
    assert_eq!(result.unwrap_err().code, "model_not_current");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        project::config(t.path()).unwrap().defaults[&Media::Image].model,
        "gpt-image-1"
    );
    assert!(
        storage::yaml::<Run>(
            &engine::run_path(t.path(), &run.id)
                .unwrap()
                .join("run.yaml")
        )
        .is_ok()
    );
}

#[test]
fn cached_discovery_is_browsable_but_cannot_authorize_execution() {
    let t = tempfile::tempdir().unwrap();
    storage::save(&t.path().join("mediaagent/cache/models.yaml"),&json!({"catalogs":[{"provider":"openai","models":[{"model":"future-image-model","media":"image","current":true,"supported":true}]}]})).unwrap();
    let list = catalog::listing(
        t.path(),
        &catalog::Filter {
            all: true,
            media: Some(Media::Image),
            ..Default::default()
        },
    );
    let model = list["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["model"] == "future-image-model")
        .unwrap();
    assert_eq!(model["current"], false);
    assert_eq!(model["supported"], false);
    assert!(
        catalog::require_current(&Selection {
            provider: Provider::Openai,
            model: "future-image-model".into()
        })
        .is_err()
    );
}
#[tokio::test]
async fn saved_legacy_operation_results_remain_readable_without_new_submissions() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("operation.yaml");
    let selection = Selection {
        provider: Provider::Openai,
        model: "gpt-6-luna".into(),
    };
    operations::perform(
        &path,
        operations::Request {
            selection: selection.clone(),
            input: json!({}),
            attempts: 1,
            retry_unknown: false,
        },
        || Ok(()),
        || async {
            Ok(operations::Output {
                value: json!({"accepted":true}),
                request_id: Some("receipt".into()),
                usage: None,
            })
        },
    )
    .await
    .unwrap();
    let mut record: Operation = storage::yaml(&path).unwrap();
    record.selection.model = "gpt-4.1-mini".into();
    storage::save(&path, &record).unwrap();
    let reused = operations::perform(
        &path,
        operations::Request {
            selection: record.selection,
            input: json!({}),
            attempts: 1,
            retry_unknown: true,
        },
        || panic!("no reserve"),
        || async { panic!("no new call") },
    )
    .await
    .unwrap();
    assert_eq!(reused.state, State::Succeeded);
    assert_eq!(reused.attempts, 1);
    assert_eq!(reused.request_id.as_deref(), Some("receipt"));
}

#[test]
fn filling_empty_credentials_preserves_export_indentation_and_comments() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join(".env"),
        "  export FISH_API_KEY= # user's note\r\n",
    )
    .unwrap();
    credentials::save_missing(
        root.path(),
        &BTreeMap::from([(Provider::Fish, "fixture'`$literal".into())]),
    )
    .unwrap();
    let written = fs::read_to_string(root.path().join(".env")).unwrap();
    assert!(written.starts_with("  export FISH_API_KEY="));
    assert!(written.ends_with(" # user's note\r\n"));
    assert_eq!(
        credentials::resolve_with("FISH_API_KEY", root.path(), |_| None).unwrap(),
        "fixture'`$literal"
    );
}
#[test]
fn concurrent_character_creation_never_overwrites_a_winning_scaffold() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().to_owned();
    let results = std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                let p = path.clone();
                scope.spawn(move || project::add_persona(&p, "unique"))
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert!(
        mediaagent::validation::character(
            &path.join("mediaagent/characters/unique/character.yaml")
        )
        .is_ok()
    );
}

#[test]
fn direct_cli_accepts_current_image_quality_and_size_controls_without_generation() {
    let root = tempfile::tempdir().unwrap();
    for quality in ["xhigh", "max"] {
        let output = cargo_bin_cmd!("mediaagent")
            .arg("--project")
            .arg(root.path())
            .args([
                "generate",
                "image",
                "--provider",
                "openai",
                "--model",
                "gpt-image-2.5-flare",
                "--prompt",
                "Offline quality preview",
                "--image-quality",
                quality,
                "--size",
                "2048x2048",
                "--dry-run",
                "--json",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(
            result["data"]["inputs"][0]["settings"]["image_quality"],
            quality
        );
        assert!(!root.path().join("mediaagent/runs").exists());
    }
}

#[test]
fn unchanged_setup_save_preserves_configuration_comments_and_formatting() {
    let root = tempfile::tempdir().unwrap();
    project::init(root.path(), &[]).unwrap();
    let path = root.path().join("mediaagent/config.yaml");
    let original = format!(
        "# Keep my configuration notes\n{}",
        fs::read_to_string(&path).unwrap()
    );
    fs::write(&path, &original).unwrap();
    setup::Session::new(project::config(root.path()).unwrap(), vec![])
        .save(root.path())
        .unwrap();
    assert_eq!(fs::read_to_string(path).unwrap(), original);
}

#[test]
fn live_bundled_catalog_hides_historical_models_unless_all_is_explicit() {
    let root = tempfile::tempdir().unwrap();
    for all in [false, true] {
        let mut command = cargo_bin_cmd!("mediaagent");
        command.arg("--project").arg(root.path()).args([
            "models",
            "--live",
            "--provider",
            "fish",
            "--json",
        ]);
        if all {
            command.arg("--all");
        }
        let output = command.assert().success().get_output().stdout.clone();
        let result: Value = serde_json::from_slice(&output).unwrap();
        let rows = result["data"]["catalogs"][0]["models"].as_array().unwrap();
        assert!(rows.iter().any(|m| m["model"] == "s2.1-pro"));
        assert_eq!(rows.iter().any(|m| m["model"] == "s1"), all);
        if !all {
            assert!(
                rows.iter()
                    .all(|m| m["current"] == true && m["supported"] == true)
            );
        }
    }
}
