//! User-local installation and bounded, checksum-verified GitHub release updates.
use crate::{
    error::{Error, Result},
    storage,
};
use async_trait::async_trait;
use fs4::fs_std::FileExt;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

pub const REPOSITORY: &str = "maxiplux/framechord";
pub const TARGET: &str = env!("FRAMECHORD_TARGET");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_BINARY: usize = 128 * 1024 * 1024;
const MAX_METADATA: usize = 1024 * 1024;
fn problem(message: impl Into<String>) -> Error {
    Error::new("installation", message, 3)
}
fn executable_name() -> &'static str {
    if cfg!(windows) {
        "framechord.exe"
    } else {
        "framechord"
    }
}
fn receipt(dir: &Path) -> PathBuf {
    dir.join(".framechord-install.yaml")
}
fn backup_path(dir: &Path) -> PathBuf {
    dir.join(if cfg!(windows) {
        ".framechord-previous.exe"
    } else {
        ".framechord-previous"
    })
}
fn backup_receipt(dir: &Path) -> PathBuf {
    dir.join(".framechord-previous.yaml")
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema_version: u32,
    version: String,
    sha256: String,
}
fn record(path: &Path, version: &str, bytes: &[u8]) -> Result<()> {
    storage::save(
        path,
        &Record {
            schema_version: 1,
            version: version.into(),
            sha256: storage::hash(bytes),
        },
    )
}
fn existing_record(path: &Path, bytes: &[u8]) -> Result<Record> {
    let r: Record = storage::yaml(path).map_err(|_| problem("Installation receipt is missing or invalid; preserve the executable and reinstall explicitly with install --force"))?;
    if r.schema_version != 1 || r.sha256 != storage::hash(bytes) {
        return Err(problem(
            "Executable does not match its installation receipt; no files were replaced",
        ));
    }
    if r.version != "unmanaged" {
        stable_version(&r.version)?;
    }
    Ok(r)
}
fn stable_version(value: &str) -> Result<Version> {
    let value = value.strip_prefix('v').unwrap_or(value);
    let v =
        Version::parse(value).map_err(|_| Error::input("Choose a stable version such as 0.6.2"))?;
    if !v.pre.is_empty() || !v.build.is_empty() {
        return Err(Error::input("Only stable release versions are supported"));
    }
    Ok(v)
}
pub fn default_bin_dir() -> Result<PathBuf> {
    if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("framechord/bin"))
    } else {
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/bin"))
    }
    .ok_or_else(|| problem("Cannot find your user directory; supply install --bin-dir PATH"))
}
fn executable_write(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| problem("Executable needs a parent directory"))?;
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    staged.write_all(bytes)?;
    staged.as_file().sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        staged
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o755))?;
    }
    if overwrite { staged.persist(path) } else { staged.persist_noclobber(path) }
        .map_err(|_| problem("Cannot replace the executable; close running framechord workers and check directory permissions. Existing backups are preserved"))?;
    Ok(())
}
fn keep_backup(executable: &Path, version: &str) -> Result<()> {
    let dir = executable.parent().unwrap();
    let bytes = fs::read(executable)?;
    executable_write(&backup_path(dir), &bytes, true)?;
    record(&backup_receipt(dir), version, &bytes)
}
/// Append one literal PATH entry, preserving the user's existing profile bytes.
pub fn configure_unix_path(bin: &Path, profile: &Path) -> Result<bool> {
    let path = bin
        .to_str()
        .ok_or_else(|| problem("PATH setup requires a UTF-8 directory; use --no-modify-path"))?;
    if path.chars().any(|c| c.is_control()) {
        return Err(Error::input(
            "Installation directory must not contain control characters",
        ));
    }
    let quoted = path.replace('\'', "'\\''");
    let entry = format!(
        "case \":$PATH:\" in *:'{quoted}':*) ;; *) export PATH='{quoted}':\"$PATH\" ;; esac"
    );
    let original = match fs::read_to_string(profile) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    if original.lines().any(|l| l == entry) {
        return Ok(false);
    }
    let mut next = original;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(&format!("\n# framechord user installation\n{entry}\n"));
    // Do not replace a symlinked shell profile with a regular file.
    let target = if profile.is_symlink() {
        fs::canonicalize(profile)?
    } else {
        profile.to_owned()
    };
    let permissions = fs::metadata(&target).ok().map(|m| m.permissions());
    storage::atomic(&target, next.as_bytes(), true)?;
    if let Some(permissions) = permissions {
        fs::set_permissions(&target, permissions)?;
    }
    Ok(true)
}
fn configure_path(bin: &Path) -> Result<Value> {
    // Windows canonical paths carry a verbatim prefix; PATH entries use ordinary paths.
    #[cfg(windows)]
    let ordinary = {
        let text = bin
            .to_str()
            .ok_or_else(|| problem("PATH setup requires a Unicode directory"))?;
        PathBuf::from(if let Some(unc) = text.strip_prefix("\\\\?\\UNC\\") {
            format!("\\\\{unc}")
        } else {
            text.strip_prefix("\\\\?\\").unwrap_or(text).to_owned()
        })
    };
    #[cfg(windows)]
    let bin = ordinary.as_path();
    if cfg!(windows) {
        let code = r#"$ErrorActionPreference='Stop'; $b=$env:FRAMECHORD_INSTALL_BIN; $p=[Environment]::GetEnvironmentVariable('Path','User'); if(-not (($p -split ';') | Where-Object { $_.TrimEnd('\') -ieq $b.TrimEnd('\') })) { [Environment]::SetEnvironmentVariable('Path', ($b+';'+$p), 'User') }"#;
        let status = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", code])
            .env("FRAMECHORD_INSTALL_BIN", bin)
            .status()?;
        if !status.success() {
            return Err(problem(
                "Executable installed, but user PATH setup failed; use the executable's full path or rerun install --no-modify-path",
            ));
        }
        return Ok(json!({"scope":"user_path","configured":true}));
    }
    let home = PathBuf::from(
        std::env::var_os("HOME").ok_or_else(|| problem("HOME is missing; use --no-modify-path"))?,
    );
    let shell = std::env::var("SHELL").unwrap_or_default();
    let profile = if shell.ends_with("/zsh") {
        std::env::var_os("ZDOTDIR")
            .map(PathBuf::from)
            .unwrap_or(home)
            .join(".zshrc")
    } else if shell.ends_with("/bash") {
        home.join(".bashrc")
    } else if shell.ends_with("/fish") {
        return Ok(
            json!({"configured":false,"message":"For fish, add the executable directory with fish_add_path, or use its full path"}),
        );
    } else {
        home.join(".profile")
    };
    let changed = configure_unix_path(bin, &profile)?;
    Ok(json!({"scope":"shell_profile","profile":profile,"configured":true,"changed":changed}))
}
pub fn install(bin_dir: Option<&Path>, modify_path: bool, force: bool) -> Result<Value> {
    let source = fs::canonicalize(std::env::current_exe()?)?;
    let bin = storage::absolute(&match bin_dir {
        Some(dir) => dir.to_owned(),
        None => default_bin_dir()?,
    })?;
    if bin.to_string_lossy().chars().any(|c| c.is_control()) {
        return Err(Error::input("Invalid installation directory"));
    }
    fs::create_dir_all(&bin)?;
    let bin = fs::canonicalize(bin)?;
    let lock = storage::lock(&bin.join(".framechord-install.lock"))?;
    lock.lock_exclusive()?;
    let destination = bin.join(executable_name());
    let bytes = fs::read(&source)?;
    let mut replaced = false;
    if destination.exists() && fs::canonicalize(&destination)? != source {
        let old = fs::read(&destination)?;
        if old != bytes {
            let old_version = match existing_record(&receipt(&bin), &old) {
                Ok(r) if r.version != "unmanaged" => r.version,
                _ if force => "unmanaged".into(),
                _ => {
                    return Err(problem(
                        "An unmanaged executable already exists at the destination; choose another --bin-dir or explicitly use --force",
                    ));
                }
            };
            keep_backup(&destination, &old_version)?;
            executable_write(&destination, &bytes, true)?;
            replaced = true;
        }
    } else if !destination.exists() {
        executable_write(&destination, &bytes, false)?;
        replaced = true;
    }
    record(&receipt(&bin), VERSION, &bytes)?;
    let path = if modify_path {
        configure_path(&bin)?
    } else {
        json!({"configured":false,"requested":false})
    };
    let message = format!(
        "{} Use the executable path above now. Then run framechord init in your project folder. Updates: framechord update; read-only check: framechord update --check. Skills and project settings are refreshed separately.",
        if path["configured"] == true {
            "Open a new terminal to use framechord on PATH."
        } else {
            "PATH was not changed; add the executable directory to PATH when wanted."
        }
    );
    Ok(
        json!({"distribution_action":"Installed framechord","version":VERSION,"target":TARGET,"executable":destination,"replaced":replaced,"path":path,"project_files_changed":false,"network_requested":false,"message":message}),
    )
}

#[async_trait]
pub trait ReleaseTransport: Send + Sync {
    async fn get(&self, url: &str, limit: usize) -> Result<Vec<u8>>;
}
struct Http {
    client: reqwest::Client,
    authorization: Option<reqwest::header::HeaderValue>,
}
fn github_authorization(
    gh_token: Option<&str>,
    github_token: Option<&str>,
) -> Result<Option<reqwest::header::HeaderValue>> {
    let selected = gh_token
        .filter(|s| !s.trim().is_empty())
        .or_else(|| github_token.filter(|s| !s.trim().is_empty()));
    selected
        .map(|token| {
            let mut header = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token.trim()))
                .map_err(|_| problem("GitHub token has invalid header characters; check GH_TOKEN or GITHUB_TOKEN. No request was sent"))?;
            header.set_sensitive(true);
            Ok(header)
        })
        .transpose()
}
impl Http {
    fn request(&self, url: &str) -> reqwest::RequestBuilder {
        let request = self.client.get(url);
        // Credentials must never reach release downloads or their CDN redirects.
        if url.starts_with(&format!(
            "https://api.github.com/repos/{REPOSITORY}/releases/"
        )) && let Some(authorization) = &self.authorization
        {
            request.header(reqwest::header::AUTHORIZATION, authorization.clone())
        } else {
            request
        }
    }
}
#[async_trait]
impl ReleaseTransport for Http {
    async fn get(&self, url: &str, limit: usize) -> Result<Vec<u8>> {
        let mut response = self.request(url).send().await.map_err(|_| {
            problem("Cannot reach GitHub Releases; check your connection and retry explicitly")
        })?;
        if !response.status().is_success() {
            if response
                .headers()
                .get("x-ratelimit-remaining")
                .is_some_and(|v| v == "0")
            {
                return Err(problem(
                    "GitHub's release API rate limit is exhausted. Wait for its reset or supply a read-only GH_TOKEN (or GITHUB_TOKEN) in the process environment; no executable was replaced",
                ));
            }
            return Err(problem(format!(
                "GitHub Releases returned HTTP {}; use the Releases page or try later",
                response.status().as_u16()
            )));
        }
        if response.content_length().is_some_and(|n| n > limit as u64) {
            return Err(problem("Release download exceeds the size limit"));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| {
            problem("Release download interrupted; the installed executable was preserved")
        })? {
            if bytes.len().saturating_add(chunk.len()) > limit {
                return Err(problem("Release download exceeds the size limit"));
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }
}
pub struct ReleaseClient {
    transport: Arc<dyn ReleaseTransport>,
}
impl ReleaseClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .https_only(true)
            .user_agent(concat!("framechord/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .map_err(|_| problem("Cannot initialize the release client"))?;
        Ok(Self {
            transport: Arc::new(Http {
                client,
                authorization: github_authorization(
                    std::env::var("GH_TOKEN").ok().as_deref(),
                    std::env::var("GITHUB_TOKEN").ok().as_deref(),
                )?,
            }),
        })
    }
    pub fn with_transport(transport: Arc<dyn ReleaseTransport>) -> Self {
        Self { transport }
    }
    pub async fn update(
        &self,
        version: Option<&str>,
        check: bool,
        force: bool,
        allow_downgrade: bool,
    ) -> Result<Value> {
        let executable = fs::canonicalize(std::env::current_exe()?)?;
        let observed_hash = if check {
            None
        } else {
            Some(storage::hash(&fs::read(&executable)?))
        };
        let suffix = match version {
            Some(v) => format!("tags/v{}", stable_version(v)?),
            None => "latest".into(),
        };
        let url = format!("https://api.github.com/repos/{REPOSITORY}/releases/{suffix}");
        let metadata = self.transport.get(&url, MAX_METADATA).await?;
        if metadata.len() > MAX_METADATA {
            return Err(problem("Release metadata exceeds the size limit"));
        }
        let release: Release =
            serde_json::from_slice(&metadata).map_err(|_| problem("Invalid release metadata"))?;
        let selected = stable_version(&release.tag_name)?;
        if release.draft
            || release.prerelease
            || version.is_some_and(|v| stable_version(v).is_ok_and(|v| v != selected))
        {
            return Err(problem(
                "Release is not the requested stable published version",
            ));
        }
        let current = stable_version(VERSION)?;
        let asset_name = format!(
            "framechord-{TARGET}{}",
            if cfg!(windows) { ".exe" } else { "" }
        );
        let asset = release.assets.into_iter().find(|a|a.name==asset_name).ok_or_else(|| problem(format!("No native binary for {TARGET} in this release; keep this executable or build from source")))?;
        let expected_url =
            format!("https://github.com/{REPOSITORY}/releases/download/v{selected}/{asset_name}");
        if asset.browser_download_url != expected_url || asset.size == 0 || asset.size > MAX_BINARY
        {
            return Err(problem("Release asset URL or size is invalid"));
        }
        let digest = asset.digest.strip_prefix("sha256:").filter(|s|s.len()==64&&s.bytes().all(|b|b.is_ascii_hexdigit())).ok_or_else(|| problem("Release asset is missing its GitHub SHA-256 digest; no executable was replaced"))?;
        if check || (selected == current && !force) || (selected < current && version.is_none()) {
            return Ok(
                json!({"distribution_action":"Release check complete","installed_version":VERSION,"available_version":selected.to_string(),"update_available":selected>current,"executable":executable,"target":TARGET,"check_only":check,"replaced":false,"message":if selected>current {"Update available. Run framechord update to install it."} else {"This executable is current or newer than the latest stable release."}}),
            );
        }
        if selected < current && !allow_downgrade {
            return Err(Error::input(
                "The selected release is older; use --allow-downgrade only for an intentional downgrade",
            ));
        }
        let dir = executable.parent().unwrap();
        let lock = storage::lock(&dir.join(".framechord-install.lock"))?;
        lock.lock_exclusive()?;
        let installed = fs::read(&executable)?;
        if observed_hash.as_deref() != Some(storage::hash(&installed).as_str())
            || (receipt(dir).exists()
                && existing_record(&receipt(dir), &installed)?.version != VERSION)
        {
            return Err(problem(
                "The installed executable changed during this update; run framechord update again from the installed version. The preceding backup was preserved",
            ));
        }
        eprintln!("Downloading framechord {selected} for {TARGET}…");
        let bytes = self.transport.get(&expected_url, MAX_BINARY).await?;
        if bytes.len() != asset.size || storage::hash(&bytes) != digest.to_lowercase() {
            return Err(problem(
                "Release checksum/size mismatch; the installed executable was preserved",
            ));
        }
        let staging = tempfile::tempdir_in(dir)?;
        let candidate = staging.path().join(executable_name());
        executable_write(&candidate, &bytes, false)?;
        verify_version(&candidate, &selected.to_string()).await?;
        keep_backup(&executable, VERSION)?;
        self_replace::self_replace(&candidate).map_err(|_| {
            problem(format!(
                "Replacement failed. Your previous executable is backed up at {}",
                backup_path(dir).display()
            ))
        })?;
        record(&receipt(dir), &selected.to_string(), &bytes)?;
        Ok(
            json!({"distribution_action":"Updated framechord","version":selected.to_string(),"previous_version":VERSION,"executable":executable,"target":TARGET,"sha256":digest,"replaced":true,"backup":backup_path(dir),"project_files_changed":false,"message":"Update complete. Existing projects and skills were preserved. Use framechord update --rollback to restore the preceding executable."}),
        )
    }
}

#[cfg(test)]
mod github_auth_tests {
    use super::*;

    #[test]
    fn optional_authentication_has_explicit_precedence_and_sensitive_headers() {
        assert!(github_authorization(None, None).unwrap().is_none());
        let selected = github_authorization(Some(" fixture-first "), Some("fixture-second"))
            .unwrap()
            .unwrap();
        assert_eq!(selected.to_str().unwrap(), "Bearer fixture-first");
        assert!(selected.is_sensitive());
        assert!(!format!("{selected:?}").contains("fixture-first"));
        assert_eq!(
            github_authorization(Some("  "), Some("fixture-second"))
                .unwrap()
                .unwrap(),
            "Bearer fixture-second"
        );
    }

    #[test]
    fn malformed_tokens_fail_without_revealing_their_values() {
        let error = github_authorization(Some("fixture-private\nvalue"), None).unwrap_err();
        assert!(!error.to_string().contains("fixture-private"));
        assert!(error.to_string().contains("No request was sent"));
    }

    #[test]
    fn authorization_is_scoped_to_this_repositorys_api_metadata() {
        let http = Http {
            client: reqwest::Client::new(),
            authorization: github_authorization(Some("fixture-scoped"), None).unwrap(),
        };
        let api = http
            .request(&format!(
                "https://api.github.com/repos/{REPOSITORY}/releases/latest"
            ))
            .build()
            .unwrap();
        assert!(
            api.headers()
                .get(reqwest::header::AUTHORIZATION)
                .unwrap()
                .is_sensitive()
        );
        for url in [
            format!("https://github.com/{REPOSITORY}/releases/download/v0.6.2/framechord"),
            "https://api.github.com.evil.invalid/repos/maxiplux/framechord/releases/latest".into(),
            "https://api.github.com/repos/another/project/releases/latest".into(),
        ] {
            assert!(
                !http
                    .request(&url)
                    .build()
                    .unwrap()
                    .headers()
                    .contains_key(reqwest::header::AUTHORIZATION)
            );
        }
    }
}
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}
#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: usize,
    #[serde(default)]
    digest: String,
}
async fn verify_version(path: &Path, expected: &str) -> Result<()> {
    let mut cmd = tokio::process::Command::new(path);
    cmd.arg("--version")
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    let output = tokio::time::timeout(Duration::from_secs(10), cmd.output())
        .await
        .map_err(|_| {
            problem("New executable version check timed out; existing executable preserved")
        })?
        .map_err(|_| {
            problem("New executable cannot run on this computer; existing executable preserved")
        })?;
    if !output.status.success()
        || String::from_utf8_lossy(&output.stdout).trim() != format!("framechord {expected}")
    {
        return Err(problem(
            "New executable version check failed; existing executable preserved",
        ));
    }
    Ok(())
}
pub async fn rollback() -> Result<Value> {
    let executable = fs::canonicalize(std::env::current_exe()?)?;
    let dir = executable.parent().unwrap();
    let lock = storage::lock(&dir.join(".framechord-install.lock"))?;
    lock.lock_exclusive()?;
    let backup = backup_path(dir);
    let bytes =
        fs::read(&backup).map_err(|_| problem("No preceding executable backup is available"))?;
    let prior = existing_record(&backup_receipt(dir), &bytes)?;
    if prior.version != "unmanaged" {
        verify_version(&backup, &prior.version).await?;
    }
    self_replace::self_replace(&backup).map_err(|_| {
        problem("Cannot restore the backup; check directory permissions and stop running workers")
    })?;
    if prior.version == "unmanaged" {
        if receipt(dir).exists() {
            fs::remove_file(receipt(dir))?;
        }
    } else {
        record(&receipt(dir), &prior.version, &bytes)?;
    }
    Ok(
        json!({"distribution_action":"Restored previous framechord","version":prior.version,"executable":executable,"project_files_changed":false,"network_requested":false,"message":"Rollback complete. Project configuration, keys, characters and skills were preserved."}),
    )
}
