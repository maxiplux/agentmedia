use assert_cmd::cargo::cargo_bin_cmd;
use async_trait::async_trait;
use mediaagent::{
    distribution::{self, ReleaseClient, ReleaseTransport},
    error::Result,
    storage,
};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, Mutex},
};
struct Mock {
    metadata: Vec<u8>,
    binary: Vec<u8>,
    calls: Mutex<Vec<String>>,
}
#[async_trait]
impl ReleaseTransport for Mock {
    async fn get(&self, url: &str, _limit: usize) -> Result<Vec<u8>> {
        self.calls.lock().unwrap().push(url.into());
        assert!(
            url.starts_with("https://api.github.com/repos/maxiplux/agentmedia/releases/")
                || url.starts_with("https://github.com/maxiplux/agentmedia/releases/download/")
        );
        Ok(if url.starts_with("https://api.") {
            self.metadata.clone()
        } else {
            self.binary.clone()
        })
    }
}
fn fixture(version: &str, bytes: &[u8]) -> Value {
    let asset = format!(
        "mediaagent-{}{}",
        distribution::TARGET,
        if cfg!(windows) { ".exe" } else { "" }
    );
    json!({"tag_name":format!("v{version}"),"draft":false,"prerelease":false,"assets":[{"name":asset,"size":bytes.len(),"browser_download_url":format!("https://github.com/maxiplux/agentmedia/releases/download/v{version}/{asset}"),"digest":format!("sha256:{}",storage::hash(bytes))}]})
}
fn transport(metadata: Value, binary: Vec<u8>) -> Arc<Mock> {
    Arc::new(Mock {
        metadata: serde_json::to_vec(&metadata).unwrap(),
        binary,
        calls: Mutex::new(vec![]),
    })
}
#[test]
fn offline_install_is_idempotent_and_does_not_require_home_for_custom_destination() {
    let root = tempfile::tempdir().unwrap();
    let bin = root.path().join("bin with spaces");
    fs::write(root.path().join(".env"), "UNRELATED=preserve\n").unwrap();
    for _ in 0..2 {
        let out = cargo_bin_cmd!("mediaagent")
            .current_dir(root.path())
            .env_remove("HOME")
            .env_remove("LOCALAPPDATA")
            .args(["install", "--bin-dir"])
            .arg(&bin)
            .args(["--no-modify-path", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let result: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(result["data"]["network_requested"], false);
        assert_eq!(result["data"]["project_files_changed"], false);
    }
    let exe = bin.join(if cfg!(windows) {
        "mediaagent.exe"
    } else {
        "mediaagent"
    });
    assert!(
        std::process::Command::new(exe)
            .arg("--version")
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        fs::read_to_string(root.path().join(".env")).unwrap(),
        "UNRELATED=preserve\n"
    );
    assert!(!root.path().join("mediaagent").exists());
}
#[test]
fn install_refuses_unmanaged_files_without_force() {
    let root = tempfile::tempdir().unwrap();
    let exe = root.path().join(if cfg!(windows) {
        "mediaagent.exe"
    } else {
        "mediaagent"
    });
    fs::write(&exe, "unrelated executable").unwrap();
    cargo_bin_cmd!("mediaagent")
        .args(["install", "--bin-dir"])
        .arg(root.path())
        .args(["--no-modify-path", "--json"])
        .assert()
        .failure();
    assert_eq!(fs::read_to_string(exe).unwrap(), "unrelated executable");
}
#[test]
fn explicit_force_install_keeps_unmanaged_bytes_without_claiming_their_version() {
    let root = tempfile::tempdir().unwrap();
    let exe = root.path().join(if cfg!(windows) {
        "mediaagent.exe"
    } else {
        "mediaagent"
    });
    fs::write(&exe, "original unmanaged file").unwrap();
    cargo_bin_cmd!("mediaagent")
        .args(["install", "--bin-dir"])
        .arg(root.path())
        .args(["--force", "--no-modify-path", "--json"])
        .assert()
        .success();
    let backup = root.path().join(if cfg!(windows) {
        ".mediaagent-previous.exe"
    } else {
        ".mediaagent-previous"
    });
    assert_eq!(
        fs::read_to_string(backup).unwrap(),
        "original unmanaged file"
    );
    let prior: Value = storage::yaml(&root.path().join(".mediaagent-previous.yaml")).unwrap();
    assert_eq!(prior["version"], "unmanaged");
}
#[test]
fn rollback_handles_an_explicitly_replaced_unmanaged_backup() {
    child_case("unmanaged-rollback");
}
#[test]
fn path_setup_preserves_content_and_quotes_literal_paths_without_duplicate_entries() {
    let root = tempfile::tempdir().unwrap();
    let profile = root.path().join(".profile");
    fs::write(&profile, "# existing settings\nexport EXISTING=preserve").unwrap();
    let bin = root.path().join("bin $literal `literal` ' quotes");
    assert!(distribution::configure_unix_path(&bin, &profile).unwrap());
    let once = fs::read(&profile).unwrap();
    assert!(!distribution::configure_unix_path(&bin, &profile).unwrap());
    assert_eq!(once, fs::read(&profile).unwrap());
    assert!(
        String::from_utf8_lossy(&once)
            .starts_with("# existing settings\nexport EXISTING=preserve\n")
    );
    #[cfg(unix)]
    {
        let result = std::process::Command::new("sh")
            .arg("-c")
            .arg(". \"$1\"; first=\"$PATH\"; . \"$1\"; test \"$first\" = \"$PATH\" || exit 1; printf '%s' \"$PATH\"")
            .arg("test")
            .arg(&profile)
            .output()
            .unwrap();
        assert!(result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stdout).starts_with(&format!("{}:", bin.display()))
        );
    }
}
#[tokio::test]
async fn check_is_read_only_and_reports_current_newer_and_unavailable_targets() {
    for (version, expected) in [(distribution::VERSION, false), ("99.0.0", true)] {
        let fake = transport(fixture(version, b"fixture"), vec![]);
        let result = ReleaseClient::with_transport(fake.clone())
            .update(None, true, false, false)
            .await
            .unwrap();
        assert_eq!(result["update_available"], expected);
        assert_eq!(fake.calls.lock().unwrap().len(), 1);
    }
    let mut metadata = fixture(distribution::VERSION, b"fixture");
    metadata["assets"] = json!([]);
    assert!(
        ReleaseClient::with_transport(transport(metadata, vec![]))
            .update(None, true, false, false)
            .await
            .is_err()
    );
}
#[tokio::test]
async fn untrusted_release_metadata_is_rejected_before_binary_download() {
    let base = fixture(distribution::VERSION, b"fixture");
    for mutate in 0..6 {
        let mut m = base.clone();
        match mutate {
            0 => m["draft"] = json!(true),
            1 => m["prerelease"] = json!(true),
            2 => m["tag_name"] = json!("../../bad"),
            3 => m["assets"][0]["browser_download_url"] = json!("https://elsewhere.invalid/asset"),
            4 => m["assets"][0]["digest"] = json!(""),
            _ => m["assets"][0]["size"] = json!(128 * 1024 * 1024 + 1),
        };
        let fake = transport(m, vec![]);
        assert!(
            ReleaseClient::with_transport(fake.clone())
                .update(None, false, true, false)
                .await
                .is_err()
        );
        assert_eq!(fake.calls.lock().unwrap().len(), 1);
    }
    let fake = transport(base, vec![]);
    assert!(
        ReleaseClient::with_transport(fake.clone())
            .update(Some("../invalid"), true, false, false)
            .await
            .is_err()
    );
    assert!(fake.calls.lock().unwrap().is_empty());
}
#[test]
fn native_update_replaces_only_an_isolated_executable_and_keeps_a_backup() {
    child_case("success");
}
#[test]
fn corrupt_update_keeps_the_executable_and_project_files_unchanged() {
    child_case("corrupt");
}
#[test]
fn wrong_version_update_is_rejected_before_replacement() {
    child_case("wrong-version");
}
#[test]
fn rollback_restores_a_verified_backup_without_network_access() {
    child_case("rollback");
}
#[test]
fn overlapping_update_preserves_the_winning_installation_and_rollback_backup() {
    child_case("overlap");
}
fn child_case(action: &str) {
    let root = tempfile::tempdir().unwrap();
    let helper = root.path().join(if cfg!(windows) {
        "test-helper.exe"
    } else {
        "test-helper"
    });
    fs::copy(std::env::current_exe().unwrap(), &helper).unwrap();
    let replacement = root.path().join(if cfg!(windows) {
        "replacement.exe"
    } else {
        "replacement"
    });
    fs::copy(env!("CARGO_BIN_EXE_mediaagent"), &replacement).unwrap();
    // Linux debug executables can exceed the production download ceiling.
    // Exercise the real executable with debug sections removed, as release assets do.
    #[cfg(target_os = "linux")]
    for path in [&helper, &replacement] {
        assert!(
            std::process::Command::new("strip")
                .arg("-S")
                .arg(path)
                .status()
                .unwrap()
                .success()
        );
    }
    let original = fs::read(&helper).unwrap();
    fs::write(root.path().join(".env"), "PRIVATE_FIXTURE=preserve\n").unwrap();
    let child = std::process::Command::new(&helper)
        .args(["--ignored", "--exact", "update_child_entry", "--nocapture"])
        .env("MEDIAAGENT_TEST_UPDATE_ACTION", action)
        .env("MEDIAAGENT_TEST_UPDATE_BINARY", &replacement)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    if matches!(
        action,
        "success" | "rollback" | "unmanaged-rollback" | "overlap"
    ) {
        assert_eq!(fs::read(&helper).unwrap(), fs::read(&replacement).unwrap());
        assert!(
            std::process::Command::new(&helper)
                .arg("--version")
                .output()
                .unwrap()
                .status
                .success()
        );
    } else {
        assert_eq!(fs::read(&helper).unwrap(), original);
    }
    assert_eq!(
        fs::read_to_string(root.path().join(".env")).unwrap(),
        "PRIVATE_FIXTURE=preserve\n"
    );
}
#[tokio::test]
#[ignore = "Only executed in an isolated copy of this test executable by the native update tests"]
async fn update_child_entry() {
    let action = std::env::var("MEDIAAGENT_TEST_UPDATE_ACTION").unwrap();
    let bytes = fs::read(std::env::var("MEDIAAGENT_TEST_UPDATE_BINARY").unwrap()).unwrap();
    let current = std::env::current_exe().unwrap();
    let dir = current.parent().unwrap();
    if action == "overlap" {
        struct Overlap {
            inner: Arc<Mock>,
            candidate: std::path::PathBuf,
        }
        #[async_trait]
        impl ReleaseTransport for Overlap {
            async fn get(&self, url: &str, limit: usize) -> Result<Vec<u8>> {
                if url.starts_with("https://api.") {
                    // Another updater completes while this process fetches metadata.
                    self_replace::self_replace(&self.candidate).unwrap();
                }
                self.inner.get(url, limit).await
            }
        }
        let backup = dir.join(if cfg!(windows) {
            ".mediaagent-previous.exe"
        } else {
            ".mediaagent-previous"
        });
        fs::write(&backup, b"preceding accepted backup").unwrap();
        let prior = dir.join(".mediaagent-previous.yaml");
        fs::write(&prior, b"preceding backup receipt").unwrap();
        let fake = transport(fixture(distribution::VERSION, &bytes), bytes);
        let result = ReleaseClient::with_transport(Arc::new(Overlap {
            inner: fake.clone(),
            candidate: std::env::var("MEDIAAGENT_TEST_UPDATE_BINARY")
                .unwrap()
                .into(),
        }))
        .update(None, false, true, false)
        .await;
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("changed during this update")
        );
        assert_eq!(fake.calls.lock().unwrap().len(), 1);
        assert_eq!(fs::read(backup).unwrap(), b"preceding accepted backup");
        assert_eq!(fs::read(prior).unwrap(), b"preceding backup receipt");
        return;
    }
    if action == "rollback" || action == "unmanaged-rollback" {
        fs::write(
            dir.join(if cfg!(windows) {
                ".mediaagent-previous.exe"
            } else {
                ".mediaagent-previous"
            }),
            &bytes,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                dir.join(".mediaagent-previous"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }
        storage::save(&dir.join(".mediaagent-previous.yaml"),&json!({"schema_version":1,"version":if action=="unmanaged-rollback" {"unmanaged"} else {distribution::VERSION},"sha256":storage::hash(&bytes)})).unwrap();
        distribution::rollback().await.unwrap();
        return;
    }
    let version = if action == "wrong-version" {
        "99.0.0"
    } else {
        distribution::VERSION
    };
    let metadata = fixture(version, &bytes);
    let body = if action == "corrupt" {
        b"corrupt".to_vec()
    } else {
        bytes
    };
    let result = ReleaseClient::with_transport(transport(metadata, body))
        .update(None, false, true, false)
        .await;
    if action == "success" {
        assert_eq!(result.unwrap()["replaced"], true);
        assert!(
            dir.join(if cfg!(windows) {
                ".mediaagent-previous.exe"
            } else {
                ".mediaagent-previous"
            })
            .is_file()
        );
    } else {
        assert!(result.is_err());
    }
}
