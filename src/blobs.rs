//! Hash-addressed reference storage; durable YAML remains the source of truth.
use crate::{
    error::{Error, Result},
    storage,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn directory(path: &Path) -> Option<PathBuf> {
    let runs = path.parent()?.parent()?;
    (path.file_name()? == "run.yaml" && runs.file_name()? == "runs")
        .then(|| runs.parent().unwrap().join("blobs"))
}
pub fn valid_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}
fn visit(
    value: &mut Value,
    f: &mut impl FnMut(&mut serde_json::Map<String, Value>) -> Result<()>,
) -> Result<()> {
    match value {
        Value::Object(o) if o.contains_key("mime") && o.contains_key("sha256") => f(o)?,
        Value::Object(o) => {
            for v in o.values_mut() {
                visit(v, f)?
            }
        }
        Value::Array(a) => {
            for v in a {
                visit(v, f)?
            }
        }
        _ => {}
    };
    Ok(())
}
pub fn store(path: &Path, value: &mut Value) -> Result<()> {
    let Some(dir) = directory(path) else {
        return Ok(());
    };
    if value["schema_version"] != 2 {
        return Ok(());
    }
    visit(value, &mut |o| {
        let Some(encoded) = o.get("base64").and_then(Value::as_str) else {
            return Ok(());
        };
        let bytes = STANDARD
            .decode(encoded)
            .map_err(|_| Error::input("Invalid reference encoding"))?;
        let hash = storage::hash(&bytes);
        if o.get("sha256").and_then(Value::as_str) != Some(&hash) {
            return Err(Error::input("Reference content hash mismatch"));
        }
        let target = dir.join(&hash);
        if target.exists() {
            if storage::hash(&fs::read(&target)?) != hash {
                return Err(Error::input("Corrupt stored reference blob"));
            }
        } else {
            storage::create(&target, &bytes)?;
        }
        o.remove("base64");
        o.insert("blob".into(), Value::String(hash));
        Ok(())
    })
}
pub fn hydrate(path: &Path, value: &mut Value) -> Result<()> {
    let Some(dir) = directory(path) else {
        return Ok(());
    };
    visit(value, &mut |o| {
        if let Some(blob) = o.remove("blob") {
            let hash = blob
                .as_str()
                .filter(|s| valid_hash(s))
                .ok_or_else(|| Error::input("Invalid blob hash"))?;
            if o.get("sha256").and_then(Value::as_str) != Some(hash) {
                return Err(Error::input("Blob/reference hash mismatch"));
            }
            let path = dir.join(hash);
            if fs::symlink_metadata(&path)?.file_type().is_symlink() {
                return Err(Error::input("Reference blob cannot be a symlink"));
            }
            let bytes = fs::read(path)?;
            if storage::hash(&bytes) != hash {
                return Err(Error::input("Corrupt reference blob"));
            }
            o.insert("base64".into(), Value::String(STANDARD.encode(bytes)));
        }
        Ok(())
    })
}
/// Small metadata projection; excludes scripts, prompts and reference bytes.
pub fn metadata(value: &Value) -> Value {
    serde_json::json!({"id":value["id"],"state":value["state"],"jobs":value["jobs"].as_array().into_iter().flatten().map(|j|serde_json::json!({"input":{"id":j["input"]["id"],"selection":j["input"]["selection"],"settings":j["input"]["settings"],"locale":j["input"]["locale"]},"fingerprint":j["fingerprint"],"state":j["state"],"artifacts":j["artifacts"]})).collect::<Vec<_>>()})
}
pub fn read_metadata(path: &Path) -> Result<Value> {
    let bytes = fs::read(path)?;
    let hash = storage::hash(&bytes);
    let cache = path.with_file_name("metadata.json");
    if let Ok(bytes) = fs::read(&cache)
        && let Ok(v) = serde_json::from_slice::<Value>(&bytes)
        && v["source_sha256"] == hash
        && v["index_version"] == 2
    {
        return Ok(v["metadata"].clone());
    }
    let value: Value =
        serde_saphyr::from_slice(&bytes).map_err(|_| Error::input("Invalid run YAML"))?;
    Ok(metadata(&value))
}
