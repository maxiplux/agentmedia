use crate::{
    contracts::Provider,
    error::{Error, Result},
};
use std::path::Path;

pub const PROVIDERS: [Provider; 5] = [
    Provider::Google,
    Provider::Openai,
    Provider::Fish,
    Provider::Openrouter,
    Provider::Elevenlabs,
];
pub const ENV_TEMPLATE: &str =
    "GEMINI_API_KEY=\nOPENAI_API_KEY=\nFISH_API_KEY=\nOPENROUTER_API_KEY=\nELEVENLABS_API_KEY=\n";

pub fn resolve(provider: Provider, root: &Path) -> Result<String> {
    resolve_with(provider.key(), root, |key| std::env::var(key).ok())
}
pub fn resolve_with(
    key: &str,
    root: &Path,
    env: impl Fn(&str) -> Option<String>,
) -> Result<String> {
    if let Some(value) = env(key) {
        return nonempty(key, value);
    }
    let path = root.join(".env");
    if path.is_file() {
        for entry in
            dotenvy::from_path_iter(path).map_err(|_| Error::input("Cannot read project .env"))?
        {
            let (name, value) = entry.map_err(|_| Error::input("Invalid entry in project .env"))?;
            if name == key {
                return nonempty(key, value);
            }
        }
    }
    Err(Error::new(
        "missing_credentials",
        format!("Set {key} in the environment or target project's .env"),
        3,
    ))
}
fn nonempty(key: &str, value: String) -> Result<String> {
    if value.trim().is_empty() {
        Err(Error::new(
            "missing_credentials",
            format!("{key} is empty"),
            3,
        ))
    } else {
        Ok(value)
    }
}

/// Fill explicitly supplied missing keys and append absent credential names with empty values.
/// Preserve existing values, comments and unrelated entries, without copying process keys.
/// This never changes a populated key or overrides process-environment precedence.
pub fn render_missing(
    root: &Path,
    values: &std::collections::BTreeMap<Provider, String>,
) -> Result<Option<String>> {
    let path = root.join(".env");
    let original = match std::fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(_) => {
            return Err(Error::input(
                "Cannot safely read project .env; it was not changed",
            ));
        }
    };
    let mut updated = original.clone();
    for (provider, value) in values {
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(Error::input(
                "Credential must be nonempty and contain no control characters",
            ));
        }
        if resolve(*provider, root).is_ok() {
            continue;
        }
        if std::env::var_os(provider.key()).is_some() {
            return Err(Error::input(format!(
                "Unset or correct {} in the process environment first; project keys cannot override it",
                provider.key()
            )));
        }
        let entries = dotenvy::from_read_iter(original.as_bytes())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| {
                Error::input("Invalid project .env; fix its syntax without exposing its values")
            })?;
        if entries
            .iter()
            .filter(|(name, _)| name == provider.key())
            .count()
            > 1
        {
            return Err(Error::input(format!(
                "Remove duplicate {} entries from the project .env before saving",
                provider.key()
            )));
        }
        // Single-quoted segments are literal in both dotenvy and ordinary shells.
        // Close/escape/reopen around apostrophes; never execute stored content.
        let encoded = value.replace('\'', "'\\''");
        let replacement = format!("{}='{}'", provider.key(), encoded);
        let mut found = false;
        let mut next = String::new();
        for line in updated.split_inclusive('\n') {
            let content = line.trim_end_matches(['\r', '\n']);
            let entry = content
                .trim_start()
                .strip_prefix("export ")
                .unwrap_or(content.trim_start());
            if entry
                .split_once('=')
                .is_some_and(|(name, _)| name.trim() == provider.key())
            {
                next.push_str(&content[..content.len() - content.trim_start().len()]);
                if content.trim_start().starts_with("export ") {
                    next.push_str("export ");
                }
                next.push_str(&replacement);
                let tail = entry.split_once('=').unwrap().1.trim();
                let comment = tail
                    .strip_prefix("\"\"")
                    .or_else(|| tail.strip_prefix("''"))
                    .unwrap_or(tail)
                    .trim_start();
                if comment.starts_with('#') {
                    next.push(' ');
                    next.push_str(comment);
                }
                next.push_str(if line.ends_with("\r\n") {
                    "\r\n"
                } else if line.ends_with('\n') {
                    "\n"
                } else {
                    ""
                });
                found = true;
            } else {
                next.push_str(line);
            }
        }
        if !found {
            if !next.is_empty() && !next.ends_with('\n') {
                next.push('\n');
            }
            next.push_str(&replacement);
            next.push('\n');
        }
        updated = next;
    }
    // Parse and round-trip before changing the file. Never surface parser snippets.
    let parsed = dotenvy::from_read_iter(updated.as_bytes())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| Error::input("Cannot safely serialize project credentials"))?;
    for (provider, value) in values {
        if resolve(*provider, root).is_err()
            && !parsed
                .iter()
                .any(|(name, read)| name == provider.key() && read == value)
        {
            return Err(Error::input(
                "Credential did not round-trip; project .env was not changed",
            ));
        }
    }
    let newline = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    for provider in PROVIDERS {
        if !parsed.iter().any(|(name, _)| name == provider.key()) {
            if !updated.is_empty() && !updated.ends_with('\n') {
                updated.push_str(newline);
            }
            updated.push_str(provider.key());
            updated.push('=');
            updated.push_str(newline);
        }
    }
    let final_entries = dotenvy::from_read_iter(updated.as_bytes())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| {
            Error::input("Cannot safely append credential names; project .env was not changed")
        })?;
    if !final_entries.starts_with(&parsed)
        || PROVIDERS
            .iter()
            .any(|p| !final_entries.iter().any(|(name, _)| name == p.key()))
    {
        return Err(Error::input(
            "Cannot safely append credential names; project .env was not changed",
        ));
    }
    if updated != original && path.is_symlink() {
        return Err(Error::input(
            "Project .env is a symbolic link and needs changes. Edit the linked credential file explicitly or provide a project-local .env with the provider variable names. The link and target were preserved",
        ));
    }
    Ok(if updated != original {
        Some(updated)
    } else {
        None
    })
}
pub fn save_missing(
    root: &Path,
    values: &std::collections::BTreeMap<Provider, String>,
) -> Result<()> {
    if let Some(content) = render_missing(root, values)? {
        crate::storage::atomic(&root.join(".env"), content.as_bytes(), true)?;
    }
    Ok(())
}
