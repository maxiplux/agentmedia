use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::Value;
use std::fs;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
#[test]
fn windows_executable_reserves_stack_for_configured_initialization() {
    // Inspect the actual linked executable, not an environment variable or
    // build-script string. The configured-init tests below exercise that binary.
    let exe = fs::read(env!("CARGO_BIN_EXE_mediaagent")).unwrap();
    assert_eq!(&exe[..2], b"MZ");
    let pe = u32::from_le_bytes(exe[0x3c..0x40].try_into().unwrap()) as usize;
    assert_eq!(&exe[pe..pe + 4], b"PE\0\0");
    let optional = pe + 24;
    let magic = u16::from_le_bytes(exe[optional..optional + 2].try_into().unwrap());
    let reserve = match magic {
        // SizeOfStackReserve in the PE32 and PE32+ optional headers.
        0x10b => u64::from(u32::from_le_bytes(
            exe[optional + 72..optional + 76].try_into().unwrap(),
        )),
        0x20b => u64::from_le_bytes(exe[optional + 72..optional + 80].try_into().unwrap()),
        _ => panic!("Unexpected PE optional header: {magic:#x}"),
    };
    assert!(reserve >= 8 * 1024 * 1024, "Main stack reserve: {reserve}");
}

#[test]
fn init_complete_defaults_is_offline_and_repeatable() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("project with spaces");
    let defaults = temporary.path().join("defaults.yaml");
    fs::write(
        &defaults,
        include_str!("../assets/templates/project-config.yaml"),
    )
    .unwrap();
    for _ in 0..2 {
        let output = cargo_bin_cmd!("mediaagent")
            .args(["--json", "--non-interactive", "init"])
            .arg(&root)
            .arg("--defaults-file")
            .arg(&defaults)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let response: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(response["data"]["generation"], false);
        assert_eq!(response["data"]["setup_required"], true);
        assert_eq!(
            response["data"]["configured_media"]
                .as_array()
                .unwrap()
                .len(),
            6
        );
    }
    assert_eq!(
        fs::read_dir(root.join("mediaagent/runs")).unwrap().count(),
        0
    );
    assert_eq!(
        fs::read_to_string(root.join(".env")).unwrap(),
        mediaagent::credentials::ENV_TEMPLATE
    );
}

#[test]
fn agent_help_and_parse_errors_are_readable() {
    let output = cargo_bin_cmd!("mediaagent")
        .args(["generate", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("--output-dir"));
    assert!(text.contains("--dry-run"));
    let out = cargo_bin_cmd!("mediaagent")
        .args(["--json", "generate", "speech", "--bad-flag"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["errors"][0]["code"], "invalid_input");
}
#[test]
fn quick_mode_dry_run_requires_no_project_and_has_exact_output() {
    let t = tempfile::tempdir().unwrap();
    let dest = t.path().join("result.png");
    let out = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(t.path())
        .args([
            "--json",
            "generate",
            "image",
            "--prompt",
            "A small forest",
            "--provider",
            "openai",
            "--model",
            "gpt-image-1",
            "--output",
        ])
        .arg(&dest)
        .arg("--dry-run")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(out.stderr.is_empty());
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        result["data"]["inputs"][0]["output"],
        dest.to_str().unwrap()
    );
    assert!(!t.path().join("mediaagent").exists());
}
#[test]
fn yaml_and_direct_requests_use_the_same_validator() {
    let t = tempfile::tempdir().unwrap();
    fs::write(t.path().join("brief.md"), "A forest").unwrap();
    fs::write(t.path().join("job.yaml"),"schema_version: 1\nid: forest\nmedia: image\nselection:\n  provider: openai\n  model: gpt-image-1\nbrief: brief.md\n").unwrap();
    let out = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(t.path())
        .args(["--json", "validate", "--kind", "job"])
        .arg(t.path().join("job.yaml"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let validated: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(validated["data"]["effective"]["prompt"], "A forest");
}
#[test]
fn json_missing_credentials_does_not_attempt_live_generation() {
    let t = tempfile::tempdir().unwrap();
    let out = cargo_bin_cmd!("mediaagent")
        .env_remove("OPENAI_API_KEY")
        .arg("--project")
        .arg(t.path())
        .args([
            "--json",
            "generate",
            "image",
            "--prompt",
            "A forest",
            "--provider",
            "openai",
            "--model",
            "gpt-image-2.5-flare",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(result["data"]["jobs"][0]["state"], "failed");
    assert!(
        result["data"]["jobs"][0]["error"]
            .as_str()
            .unwrap()
            .contains("missing_credentials")
    );
    assert!(out.stderr.is_empty());
}

#[test]
fn job_overrides_preserve_authored_input_and_unmentioned_settings() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("brief.md"), "Exact authored image brief").unwrap();
    let original = "schema_version: 1\nid: overridden\nmedia: image\nselection:\n  provider: openai\n  model: gpt-image-1\nbrief: brief.md\nsettings:\n  image_quality: medium\n  size: 1024x1024\n";
    let job = root.path().join("job.yaml");
    fs::write(&job, original).unwrap();
    let out = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(root.path())
        .args(["--json", "generate", "--job"])
        .arg(&job)
        .args([
            "--provider",
            "openai",
            "--model",
            "gpt-image-1.5",
            "--image-quality",
            "low",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let response: Value = serde_json::from_slice(&out.stdout).unwrap();
    let effective = &response["data"]["inputs"][0];
    assert_eq!(effective["selection"]["model"], "gpt-image-1.5");
    assert_eq!(effective["settings"]["image_quality"], "low");
    assert_eq!(effective["settings"]["size"], "1024x1024");
    assert_eq!(effective["prompt"], "Exact authored image brief");
    assert_eq!(fs::read_to_string(job).unwrap(), original);
}

#[test]
fn vocal_and_instrumental_options_are_explicit_and_reject_wrong_media() {
    let root = tempfile::tempdir().unwrap();
    let out = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(root.path())
        .args([
            "--json",
            "generate",
            "music",
            "--provider",
            "elevenlabs",
            "--model",
            "music_v1",
            "--prompt",
            "Sing the exact lyrics: Hello world",
            "--duration-seconds",
            "3",
            "--vocal",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let response: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        response["data"]["inputs"][0]["settings"]["instrumental"],
        false
    );
    let out = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(root.path())
        .args([
            "--json",
            "generate",
            "image",
            "--provider",
            "openai",
            "--model",
            "gpt-image-1",
            "--prompt",
            "A forest",
            "--instrumental",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn referenced_job_worker_startup_is_confirmed_without_lock_contention() {
    let root = tempfile::tempdir().unwrap();
    let mut image = image::RgbImage::new(512, 512);
    let mut random = 1u32;
    for pixel in image.pixels_mut() {
        for channel in &mut pixel.0 {
            random ^= random << 13;
            random ^= random >> 17;
            random ^= random << 5;
            *channel = random as u8;
        }
    }
    let reference = root.path().join("reference.png");
    image.save(&reference).unwrap();
    for _ in 0..2 {
        let out = cargo_bin_cmd!("mediaagent")
            .env_remove("OPENAI_API_KEY")
            .arg("--project")
            .arg(root.path())
            .args([
                "--json",
                "generate",
                "image",
                "--provider",
                "openai",
                "--model",
                "gpt-image-2.5-flare",
                "--prompt",
                "Reuse this reference",
                "--fresh",
                "--reference",
            ])
            .arg(&reference)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(3));
        let response: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            response["errors"][0]["code"], "operation_incomplete",
            "{response}"
        );
        assert_eq!(response["data"]["jobs"][0]["attempts"], 1);
        assert!(
            response["data"]["jobs"][0]["error"]
                .as_str()
                .unwrap()
                .contains("missing_credentials")
        );
        let saved_reference = &response["data"]["jobs"][0]["input"]["references"][0];
        assert!(saved_reference.get("base64").is_none());
        assert!(saved_reference["sha256"].is_string());
        assert!(!response["warnings"].as_array().unwrap().is_empty());
        assert!(out.stdout.len() < 10_000);
    }
}

#[test]
fn voice_design_dry_run_preserves_character_and_creates_no_run() {
    use mediaagent::{contracts::*, project, storage};
    let root = tempfile::tempdir().unwrap();
    let path = project::add_persona(root.path(), "narrator").unwrap();
    let mut character: Character = storage::yaml(&path).unwrap();
    character.voice = Some(Voice {
        samples: vec![],
        description: "voice.md".into(),
        selection: Selection {
            provider: Provider::Elevenlabs,
            model: "eleven_flash_v2_5".into(),
        },
        binding: None,
        references: vec![],
    });
    storage::save(&path, &character).unwrap();
    let original = fs::read(&path).unwrap();
    let script = root.path().join("preview.md");
    let words = "Hello, this exact script is for a fictional narrator preview. ".repeat(3);
    fs::write(&script, &words).unwrap();
    let out = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(root.path())
        .args([
            "--json",
            "persona",
            "voice-design",
            "narrator",
            "--model",
            "eleven_ttv_v3",
            "--text-file",
        ])
        .arg(&script)
        .args(["--dry-run"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let response: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(response["data"]["design"]["text"], words);
    assert_eq!(response["data"]["provider_voice_created"], false);
    assert!(!root.path().join("mediaagent/runs").exists());
    assert_eq!(fs::read(path).unwrap(), original);
}

#[test]
fn catalog_builtin_and_noninteractive_wizard_contracts() {
    let root = tempfile::tempdir().unwrap();
    let output = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(root.path())
        .args(["voices", "--provider", "openai", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let data: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(data["data"]["built_in_voices"][0], "alloy");
    let output = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(root.path())
        .args(["config", "wizard", "--non-interactive", "--json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!root.path().join("mediaagent/config.yaml").exists());
}

#[test]
fn identity_reference_cli_and_yaml_resolve_the_same_effective_contract() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("identity.png");
    let mut bytes = std::io::Cursor::new(vec![]);
    image::RgbImage::from_pixel(2, 2, image::Rgb([20, 30, 40]))
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    fs::write(&file, bytes.into_inner()).unwrap();
    let output = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(root.path())
        .args([
            "generate",
            "image",
            "--provider",
            "openai",
            "--model",
            "gpt-image-1",
            "--prompt",
            "Same character in a forest.",
            "--identity-reference",
        ])
        .arg(&file)
        .args(["--dry-run", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        response["data"]["inputs"][0]["references"][0]["role"],
        "identity"
    );
    assert_eq!(
        response["data"]["inputs"][0]["settings"]["input_fidelity"],
        "high"
    );
    assert!(!root.path().join("mediaagent/runs").exists());
}

#[test]
fn init_defaults_file_configures_profiles_and_dry_run_without_duplicate_flags() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("project");
    let config = t.path().join("defaults.yaml");
    fs::write(&config,"schema_version: 1\ndefaults:\n  image: {provider: openai, model: gpt-image-1}\ndefault_settings:\n  image: {format: png, size: 1536x1024, image_quality: high, input_fidelity: high}\nconsistency: {require_known_identity_model: true, inherit_source_settings: true}\n").unwrap();
    cargo_bin_cmd!("mediaagent")
        .args(["--json", "--non-interactive", "init"])
        .arg(&root)
        .arg("--defaults-file")
        .arg(&config)
        .assert()
        .success();
    let out = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(&root)
        .args([
            "--json",
            "generate",
            "image",
            "--prompt",
            "A new original",
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        value["data"]["inputs"][0]["selection"]["model"],
        "gpt-image-1"
    );
    assert_eq!(value["data"]["inputs"][0]["settings"]["size"], "1536x1024");
    assert!(value["data"]["inputs"][0]["settings"]["input_fidelity"].is_null());
    let character = mediaagent::validation::character(
        &root.join("mediaagent/characters/persona/character.yaml"),
    )
    .unwrap();
    assert_eq!(character.visual.unwrap().selection.model, "gpt-image-1");
    assert!(
        !root.join("mediaagent/runs").read_dir().unwrap().any(|e| e
            .unwrap()
            .path()
            .join("run.yaml")
            .exists())
    );
}
#[test]
fn init_interactive_rejects_automation_and_nonterminal_without_partial_writes() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("unwritten");
    for options in [vec!["--json"], vec!["--non-interactive"], vec![]] {
        let out = cargo_bin_cmd!("mediaagent")
            .args(options)
            .arg("init")
            .arg(&root)
            .arg("--interactive")
            .output()
            .unwrap();
        assert!(!out.status.success());
        assert!(!root.exists());
    }
}

#[test]
fn doctor_missing_keys_and_environment_precedence_need_no_network_or_initialization() {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join(".env"),
        "OPENAI_API_KEY=LOCAL_TEST_VALUE\n",
    )
    .unwrap();
    for (live, expected) in [(false, 0), (true, 3)] {
        let mut command = cargo_bin_cmd!("mediaagent");
        command
            .env("OPENAI_API_KEY", "")
            .arg("--project")
            .arg(root.path())
            .args(["doctor", "--provider", "openai", "--json"]);
        if live {
            command.arg("--live");
        }
        let out = command.output().unwrap();
        assert_eq!(out.status.code(), Some(expected));
        assert!(out.stderr.is_empty());
        let result: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(result["data"]["network_checked"], false);
        assert_eq!(
            result["data"]["credentials"][0]["status"],
            "missing_credentials"
        );
        assert_eq!(result["data"]["credentials"][0]["configured"], false);
        assert!(!String::from_utf8_lossy(&out.stdout).contains("LOCAL_TEST_VALUE"));
        assert!(!root.path().join("mediaagent").exists());
    }
    let out = cargo_bin_cmd!("mediaagent")
        .arg("--project")
        .arg(root.path())
        .env("OPENAI_API_KEY", "bad\nheader")
        .args(["doctor", "--live", "--provider", "openai", "--json"])
        .output()
        .unwrap();
    let result: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        result["data"]["credentials"][0]["status"],
        "invalid_configuration"
    );
    assert_eq!(result["data"]["network_checked"], false);
    cargo_bin_cmd!("mediaagent")
        .args(["doctor", "--timeout-seconds", "0", "--json"])
        .assert()
        .code(2);
}
