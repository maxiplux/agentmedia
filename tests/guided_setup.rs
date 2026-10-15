use assert_cmd::cargo::cargo_bin_cmd;
use framechord::{catalog, contracts::*, credentials, engine, operations, project, setup, storage};
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
fn init_reports_blocking_executable_before_writing_and_preserves_all_existing_files() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("framechord"), "fixture executable").unwrap();
    fs::write(root.path().join(".env"), "OTHER=keep-existing\n").unwrap();
    let output = cargo_bin_cmd!("framechord")
        .current_dir(root.path())
        .args(["init", "--json", "--non-interactive"])
        .assert()
        .code(2)
        .get_output()
        .stdout
        .clone();
    let result: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(result["errors"][0]["code"], "init_path_conflict");
    assert!(
        result["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("framechord init ./my-project")
    );
    assert_eq!(
        result["errors"][0]["corrective_argv"],
        json!(["framechord", "init", "./my-project"])
    );
    assert_eq!(
        fs::read_to_string(root.path().join("framechord")).unwrap(),
        "fixture executable"
    );
    assert_eq!(
        fs::read_to_string(root.path().join(".env")).unwrap(),
        "OTHER=keep-existing\n"
    );
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
    assert!(
        setup::Session::new(setup::recommended(), vec![])
            .save(root.path())
            .is_err()
    );
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 2);
}

#[test]
fn init_honors_global_project_and_explicit_destination_with_executable_beside_it() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("framechord"), "fixture executable").unwrap();
    cargo_bin_cmd!("framechord")
        .current_dir(root.path())
        .args([
            "--project",
            "selected project",
            "init",
            "--json",
            "--non-interactive",
        ])
        .assert()
        .success();
    assert!(
        root.path()
            .join("selected project/framechord/config.yaml")
            .is_file()
    );
    cargo_bin_cmd!("framechord")
        .current_dir(root.path())
        .args([
            "--project",
            "unused",
            "init",
            "explicit project",
            "--json",
            "--non-interactive",
        ])
        .assert()
        .success();
    assert!(
        root.path()
            .join("explicit project/framechord/config.yaml")
            .is_file()
    );
    assert!(!root.path().join("unused").exists());
    assert_eq!(
        fs::read_to_string(root.path().join("framechord")).unwrap(),
        "fixture executable"
    );
}

#[test]
fn init_rejects_project_paths_with_a_blocking_file_or_ancestor_without_mutations() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("blocking-file");
    fs::write(&file, "keep").unwrap();
    for target in [&file, &file.join("child")] {
        let error = project::init(target, &[]).unwrap_err();
        assert_eq!(error.code, "init_path_conflict");
    }
    assert_eq!(fs::read_to_string(file).unwrap(), "keep");
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
}

#[test]
fn recommended_init_and_status_are_noninteractive_offline_and_resumable() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("project with spaces");
    let result = cargo_bin_cmd!("framechord")
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
        "gemini-nano-banana-2.1"
    );
    assert_eq!(
        v["data"]["configuration"]["defaults"]["speech"]["provider"],
        "google"
    );
    assert_eq!(v["data"]["setup_complete"], false);
    assert!(
        v["data"]["pending_steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.as_str().unwrap().contains("GEMINI_API_KEY"))
    );
    assert_eq!(
        fs::read_to_string(root.join(".env")).unwrap(),
        credentials::ENV_TEMPLATE
    );
    let before = fs::read(root.join("framechord/config.yaml")).unwrap();
    cargo_bin_cmd!("framechord")
        .args(["--non-interactive", "init"])
        .arg(&root)
        .args(["--preset", "empty"])
        .assert()
        .success();
    assert_eq!(
        before,
        fs::read(root.join("framechord/config.yaml")).unwrap()
    );
    let shown = cargo_bin_cmd!("framechord")
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
fn google_and_openai_presets_are_ready_with_one_key_and_preserve_existing_identity() {
    let temporary = tempfile::tempdir().unwrap();
    for (preset, provider, key, voice, features) in [
        ("recommended", Provider::Google, "GEMINI_API_KEY", "Kore", 6),
        ("google", Provider::Google, "GEMINI_API_KEY", "Kore", 6),
        ("openai", Provider::Openai, "OPENAI_API_KEY", "coral", 5),
    ] {
        let root = temporary.path().join(preset);
        fs::create_dir(&root).unwrap();
        fs::write(
            root.join(".env"),
            format!("# Keep my key\n{key}=fixture-private\n"),
        )
        .unwrap();
        let output = cargo_bin_cmd!("framechord")
            .arg("--project")
            .arg(&root)
            .args(["init", "--preset", preset, "--json", "--non-interactive"])
            .env_remove("GEMINI_API_KEY")
            .env_remove("OPENAI_API_KEY")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        assert!(!String::from_utf8_lossy(&output).contains("fixture-private"));
        let report: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(report["data"]["setup_complete"], true);
        assert_eq!(
            report["data"]["skills"],
            json!({"codex":true,"claude":true,"cursor":true,"gemini":true,"opencode":true,"copilot":true})
        );
        let config = project::config(&root).unwrap();
        assert_eq!(config.defaults.len(), features);
        assert!(config.defaults.values().all(|s| s.provider == provider));
        assert!(!config.defaults.contains_key(&Media::Sfx));
        for media in [Media::Speech, Media::Podcast] {
            assert_eq!(
                config.default_settings[&media].voice.as_deref(),
                Some(voice)
            );
        }
        let character_path = project::character_path(&root, "persona").unwrap();
        let character: Character = storage::yaml(&character_path).unwrap();
        assert_eq!(
            character.visual.as_ref().unwrap().selection.provider,
            provider
        );
        assert_eq!(
            character.voice.as_ref().unwrap().binding.as_deref(),
            Some(voice)
        );
        let original = fs::read(&character_path).unwrap();
        let env = fs::read(root.join(".env")).unwrap();
        cargo_bin_cmd!("framechord")
            .arg("--project")
            .arg(&root)
            .args([
                "init",
                "--preset",
                if provider == Provider::Google {
                    "openai"
                } else {
                    "google"
                },
                "--json",
                "--non-interactive",
            ])
            .assert()
            .success();
        assert_eq!(fs::read(character_path).unwrap(), original);
        assert_eq!(fs::read(root.join(".env")).unwrap(), env);
        assert_eq!(project::config(&root).unwrap().defaults, config.defaults);
    }
}
#[test]
fn init_updates_only_target_env_and_never_copies_process_or_ancestor_credentials() {
    let temporary = tempfile::tempdir().unwrap();
    fs::write(
        temporary.path().join(".env"),
        "GEMINI_API_KEY=ancestor-private\n",
    )
    .unwrap();
    let root = temporary.path().join("called here");
    fs::create_dir(&root).unwrap();
    let prefix = "# Preserve comments\r\nOTHER=value\r\nexport GEMINI_API_KEY='project-private'\r\nOPENAI_API_KEY=\r\n";
    fs::write(root.join(".env"), prefix).unwrap();
    for _ in 0..2 {
        let output = cargo_bin_cmd!("framechord")
            .current_dir(&root)
            .env_remove("GEMINI_API_KEY")
            .env("OPENAI_API_KEY", "process-private")
            .args(["init", "--json", "--non-interactive"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let serialized = String::from_utf8_lossy(&output);
        for private in ["ancestor-private", "project-private", "process-private"] {
            assert!(!serialized.contains(private));
        }
        let saved = fs::read_to_string(root.join(".env")).unwrap();
        assert_eq!(
            saved,
            format!("{prefix}FISH_API_KEY=\r\nOPENROUTER_API_KEY=\r\nELEVENLABS_API_KEY=\r\n")
        );
        assert_eq!(
            credentials::resolve_with("OPENAI_API_KEY", &root, |_| Some("process-private".into()))
                .unwrap(),
            "process-private"
        );
    }
    let target = root.join("different target");
    cargo_bin_cmd!("framechord")
        .current_dir(&root)
        .arg("--project")
        .arg(&target)
        .args(["init", "--json", "--non-interactive"])
        .env("GEMINI_API_KEY", "process-private")
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(target.join(".env")).unwrap(),
        credentials::ENV_TEMPLATE
    );
    assert_eq!(
        fs::read_to_string(temporary.path().join(".env")).unwrap(),
        "GEMINI_API_KEY=ancestor-private\n"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(target.join(".env"))
                .unwrap()
                .permissions()
                .mode()
                & 0o077,
            0
        );
    }
}
#[test]
fn malformed_env_stops_init_before_project_or_skill_writes_without_secret_output() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    let content = "GEMINI_API_KEY=\"fixture-secret-unclosed\n";
    fs::write(root.join(".env"), content).unwrap();
    let result = cargo_bin_cmd!("framechord")
        .current_dir(root)
        .args(["init", "--json", "--non-interactive"])
        .assert()
        .failure()
        .get_output()
        .clone();
    assert!(!String::from_utf8_lossy(&result.stdout).contains("fixture-secret"));
    assert!(!String::from_utf8_lossy(&result.stderr).contains("fixture-secret"));
    assert_eq!(fs::read_to_string(root.join(".env")).unwrap(), content);
    assert_eq!(fs::read_dir(root).unwrap().count(), 1);
}
#[test]
fn init_waits_for_setup_lock_and_preserves_credentials_saved_while_waiting() {
    use fs4::fs_std::FileExt;
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    project::init(root, &[]).unwrap();
    fs::write(root.join(".env"), "GEMINI_API_KEY=\n").unwrap();
    let guard = storage::lock(&root.join("framechord/setup.lock")).unwrap();
    guard.lock_exclusive().unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_framechord"))
        .current_dir(root)
        .args(["init", "--json", "--non-interactive"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "Init must wait for the credential writer"
    );
    let prefix = "# Concurrent reviewed save\nGEMINI_API_KEY=fixture-new-private\n";
    fs::write(root.join(".env"), prefix).unwrap();
    drop(guard);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("fixture-new-private"));
    let saved = fs::read_to_string(root.join(".env")).unwrap();
    assert!(saved.starts_with(prefix));
    assert_eq!(saved.matches("GEMINI_API_KEY=").count(), 1);
}
#[cfg(unix)]
#[test]
fn init_preserves_linked_credentials_and_reads_a_complete_link_without_replacing_it() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("project");
    fs::create_dir(&root).unwrap();
    let target = temporary.path().join("managed-credentials");
    let prefix = "GEMINI_API_KEY=fixture-private-linked\n";
    fs::write(&target, prefix).unwrap();
    std::os::unix::fs::symlink(&target, root.join(".env")).unwrap();
    let error = project::init(&root, &[]).unwrap_err();
    assert!(error.to_string().contains("symbolic link"));
    assert!(!error.to_string().contains("fixture-private"));
    assert!(root.join(".env").is_symlink());
    assert_eq!(fs::read(&target).unwrap(), prefix.as_bytes());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    assert!(
        credentials::save_missing(
            &root,
            &BTreeMap::from([(Provider::Fish, "fixture-other-private".into())])
        )
        .is_err()
    );
    assert_eq!(fs::read(&target).unwrap(), prefix.as_bytes());
    let complete = credentials::ENV_TEMPLATE
        .replace("GEMINI_API_KEY=", "GEMINI_API_KEY=fixture-private-linked");
    fs::write(&target, &complete).unwrap();
    project::init_with_config(&root, &[], Some(&setup::recommended())).unwrap();
    assert!(root.join(".env").is_symlink());
    assert_eq!(fs::read_to_string(&target).unwrap(), complete);
    assert_eq!(setup::status(&root).unwrap()["setup_complete"], true);
}
#[test]
fn readiness_requires_voice_and_keys_and_preserves_explicitly_disabled_features() {
    let t = tempfile::tempdir().unwrap();
    let mut s = setup::Session::new(setup::recommended(), vec!["codex".into(), "claude".into()]);
    assert!(!s.finishable(t.path()));
    s.config.defaults.remove(&Media::Music);
    s.config.default_settings.remove(&Media::Music);
    for media in [Media::Speech, Media::Podcast] {
        assert_eq!(
            s.config.default_settings[&media].voice.as_deref(),
            Some("Kore")
        );
        s.config.default_settings.get_mut(&media).unwrap().voice = None;
    }
    fs::write(t.path().join(".env"), "GEMINI_API_KEY=fixture-google\n").unwrap();
    assert!(!s.finishable(t.path()));
    for media in [Media::Speech, Media::Podcast] {
        s.config.default_settings.get_mut(&media).unwrap().voice = Some("fixture-voice".into());
    }
    assert!(s.finishable(t.path()));
    let saved = s.save(t.path()).unwrap();
    assert_eq!(saved["setup_complete"], true);
    assert_eq!(saved["account_access_verified"], false);
    assert!(!s.config.defaults.contains_key(&Media::Music));
    assert!(t.path().join(".agents/skills/framechord/SKILL.md").exists());
    assert!(t.path().join(".claude/skills/framechord/SKILL.md").exists());
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
impl framechord::providers::Adapter for Count {
    async fn generate(
        &self,
        _: &Effective,
    ) -> framechord::error::Result<framechord::providers::Generated> {
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
    storage::save(&t.path().join("framechord/cache/models.yaml"),&json!({"catalogs":[{"provider":"openai","models":[{"model":"future-image-model","media":"image","current":true,"supported":true}]}]})).unwrap();
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
    assert!(written.lines().next().unwrap().ends_with(" # user's note"));
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
        framechord::validation::character(
            &path.join("framechord/characters/unique/character.yaml")
        )
        .is_ok()
    );
}

#[test]
fn direct_cli_accepts_current_image_quality_and_size_controls_without_generation() {
    let root = tempfile::tempdir().unwrap();
    for quality in ["xhigh", "max"] {
        let output = cargo_bin_cmd!("framechord")
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
        assert!(!root.path().join("framechord/runs").exists());
    }
}

#[test]
fn unchanged_setup_save_preserves_configuration_comments_and_formatting() {
    let root = tempfile::tempdir().unwrap();
    project::init(root.path(), &[]).unwrap();
    let path = root.path().join("framechord/config.yaml");
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
        let mut command = cargo_bin_cmd!("framechord");
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

#[test]
fn status_reports_shared_discovery_for_every_known_agent() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    project::init(root, &["codex".into()]).unwrap();
    let status = setup::status(root).unwrap();
    assert_eq!(
        status["skills"],
        json!({"codex":true,"claude":false,"cursor":true,"gemini":true,"opencode":true,"copilot":true})
    );
    let targets = status["skill_targets"].as_array().unwrap();
    assert_eq!(targets.len(), 6);
    let cursor = targets.iter().find(|t| t["agent"] == "cursor").unwrap();
    assert_eq!(cursor["label"], "Cursor");
    assert_eq!(cursor["native_installed"], false);
    // JSON strings, so the separators matter: every component is joined on its own.
    assert_eq!(
        cursor["native_path"],
        json!(root.join(".cursor").join("skills").join("framechord"))
    );
    assert_eq!(
        cursor["discovered_via"],
        json!([root.join(".agents").join("skills").join("framechord")])
    );
    assert!(cursor["source"].as_str().unwrap().starts_with("https://"));
    let shared = status["skills_shared_by"][".agents/skills"]
        .as_array()
        .unwrap();
    assert!(shared.iter().any(|a| a == "copilot"), "{shared:?}");
    assert!(shared.iter().any(|a| a == "codex"));
    assert!(!shared.iter().any(|a| a == "claude"));
}

#[test]
fn skill_paths_use_native_separators_for_installer_and_report() {
    use std::path::{MAIN_SEPARATOR, Path};
    let root = Path::new("root");
    let s = MAIN_SEPARATOR;
    let foreign = if s == '/' { '\\' } else { '/' };
    assert_eq!(
        project::skill_dir(root, ".config/opencode/skills").to_string_lossy(),
        format!("root{s}.config{s}opencode{s}skills{s}framechord")
    );
    for target in project::AGENT_TARGETS {
        for user_scope in [false, true] {
            let dirs = if user_scope {
                target.user
            } else {
                target.project
            };
            let installed = project::native_skill_dir(root, target, user_scope);
            assert_eq!(
                installed,
                project::skill_dir(root, dirs[0]),
                "{} user_scope={user_scope}",
                target.name
            );
            for dir in dirs {
                let reported = project::skill_dir(root, dir).to_string_lossy().into_owned();
                assert!(
                    !reported.contains(foreign),
                    "{} {dir}: {reported}",
                    target.name
                );
            }
        }
    }
    let t = tempfile::tempdir().unwrap();
    project::init(t.path(), &["opencode".into()]).unwrap();
    let report = project::skills_report(t.path());
    let opencode = report["skill_targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["agent"] == "opencode")
        .unwrap();
    let native = project::skill_dir(t.path(), ".opencode/skills");
    assert_eq!(opencode["native_path"], json!(native));
    assert_eq!(opencode["discovered_via"], json!([native]));
    assert!(native.join("SKILL.md").is_file());
}

#[test]
fn none_agent_target_must_be_given_alone() {
    for agents in [["none", "none"], ["none", "codex"], ["claude", "none"]] {
        let agents: Vec<String> = agents.iter().map(|a| a.to_string()).collect();
        let error = project::normalize_agents(&agents)
            .err()
            .unwrap_or_else(|| panic!("{agents:?} was accepted"));
        assert_eq!(error.code, "invalid_input", "{agents:?}");
        assert!(
            error.message.contains("none must be given alone"),
            "{agents:?}: {}",
            error.message
        );
        assert!(
            !error.message.contains("must be one of"),
            "{agents:?}: {}",
            error.message
        );
    }
    assert!(
        project::normalize_agents(&["none".into()])
            .unwrap()
            .is_empty()
    );
    let unknown = project::normalize_agents(&["nonsense".into()])
        .err()
        .expect("unknown agent is rejected");
    assert!(
        unknown.message.contains("must be one of"),
        "{}",
        unknown.message
    );
}

#[test]
fn status_flags_duplicate_discovery_and_the_human_output_warns_once() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    project::init(root, &["codex".into()]).unwrap();
    let status = setup::status(root).unwrap();
    for target in status["skill_targets"].as_array().unwrap() {
        assert_eq!(target["duplicate_discovery"], false, "{}", target["agent"]);
    }
    assert!(!framechord::cli::human_result(&status).contains("Warning"));

    project::install(root, &["cursor".into()]).unwrap();
    let status = setup::status(root).unwrap();
    let targets = status["skill_targets"].as_array().unwrap();
    let cursor = targets.iter().find(|t| t["agent"] == "cursor").unwrap();
    assert_eq!(cursor["duplicate_discovery"], true);
    assert_eq!(cursor["native_installed"], true);
    assert_eq!(cursor["discovered_via"].as_array().unwrap().len(), 2);
    let codex = targets.iter().find(|t| t["agent"] == "codex").unwrap();
    assert_eq!(codex["duplicate_discovery"], false);
    let copilot = targets.iter().find(|t| t["agent"] == "copilot").unwrap();
    assert_eq!(copilot["duplicate_discovery"], false);

    let human = framechord::cli::human_result(&status);
    let warnings: Vec<&str> = human.lines().filter(|l| l.contains("Warning")).collect();
    assert_eq!(warnings.len(), 1, "{human}");
    assert!(
        warnings[0].contains("Cursor")
            && warnings[0].contains(&*project::skill_dir(root, ".cursor/skills").to_string_lossy())
            && warnings[0].contains(&*project::skill_dir(root, ".agents/skills").to_string_lossy())
            && warnings[0].contains("keep one"),
        "{}",
        warnings[0]
    );
}
