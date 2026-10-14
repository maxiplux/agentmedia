use crate::error::{Error, Result};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub fn yaml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path)?;
    // Do not render parser snippets: manifests may contain private content.
    serde_saphyr::from_str(&content).map_err(|_| {
        Error::input(format!(
            "Invalid YAML or unknown fields in {}",
            path.display()
        ))
    })
}
pub fn atomic(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all()?;
    if overwrite {
        tmp.persist(path).map_err(|e| Error::io(e.error))?;
    } else {
        tmp.persist_noclobber(path)
            .map_err(|e| Error::io(e.error))?;
    }
    Ok(())
}
pub fn save<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let content = serde_saphyr::to_string(data)
        .map_err(|_| Error::new("serialization", "Cannot serialize local state", 1))?;
    atomic(path, content.as_bytes(), true)
}
pub fn create(path: &Path, bytes: &[u8]) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    match atomic(path, bytes, false) {
        Ok(()) => Ok(true),
        Err(e) if path.exists() => {
            let _ = e;
            Ok(false)
        }
        Err(e) => Err(e),
    }
}
pub fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}
pub fn relative(base: &Path, path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    };
    Ok(fs::canonicalize(path)?)
}
pub fn text(base: &Path, path: &Path) -> Result<String> {
    let text = fs::read_to_string(relative(base, path)?)?;
    if text.trim().is_empty() {
        return Err(Error::input(format!(
            "Required content is empty: {}",
            path.display()
        )));
    }
    Ok(text)
}
pub fn version(v: u32) -> Result<()> {
    if v == 1 {
        Ok(())
    } else {
        Err(Error::input(format!(
            "Unsupported schema_version {v}; expected 1"
        )))
    }
}
pub fn id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        Err(Error::input(
            "IDs require 1–64 lowercase letters, digits, or hyphens",
        ))
    } else {
        Ok(())
    }
}
pub fn lock(path: &Path) -> Result<fs::File> {
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| Error::input("Lock path requires parent"))?,
    )?;
    Ok(fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?)
}
