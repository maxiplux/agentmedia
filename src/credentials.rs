use crate::{
    contracts::Provider,
    error::{Error, Result},
};
use std::path::Path;

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
