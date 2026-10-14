//! Validate model-authored bundles before reading dependencies or publishing files.
use crate::{
    contracts::*,
    error::{Error, Result},
    storage::*,
    validation,
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Files {
    files: Vec<PlannedFile>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlannedFile {
    path: PathBuf,
    content: String,
}

pub fn stage(content: &str, kind: &str, directory: &Path, config: &Config) -> Result<PathBuf> {
    if !matches!(kind, "character" | "job" | "infographic") {
        return Err(Error::input("Unsupported plan kind"));
    }
    if content.len() > 1024 * 1024 {
        return Err(Error::input(
            "Planner bundle exceeds 1 MiB; raw response remains in the run",
        ));
    }
    let files: Files = serde_json::from_str(content.trim()).map_err(|_| {
        Error::input("Planner output was not valid file JSON; raw response remains in the run")
    })?;
    if files.files.is_empty() || files.files.len() > 32 {
        return Err(Error::input("Planner bundle requires 1–32 authoring files"));
    }
    let mut names = BTreeSet::new();
    // Check the whole list before creating even the first file.
    for file in &files.files {
        authoring_path(&file.path)?;
        if !names.insert(file.path.clone()) {
            return Err(Error::input("Planner emitted duplicate file paths"));
        }
        if !matches!(
            file.path.extension().and_then(|s| s.to_str()),
            Some("yaml" | "yml" | "md" | "txt")
        ) {
            return Err(Error::input(
                "Planner may author only YAML, Markdown, and text files",
            ));
        }
    }
    for file in files.files {
        atomic(&directory.join(file.path), file.content.as_bytes(), false)?;
    }
    let root = std::fs::canonicalize(directory)?;
    let manifest = root.join(format!("{kind}.yaml"));
    match kind {
        "character" => {
            contained_character(&root, &manifest)?;
            validation::character(&manifest)?;
        }
        "infographic" => {
            contained_infographic(&root, &manifest)?;
            validation::infographic(&manifest)?;
        }
        "job" => {
            let job: Job = yaml(&manifest)?;
            let base = manifest.parent().unwrap();
            for sample in &job.voice_samples {
                contained_sample(&root, base, sample)?;
            }
            for path in [&job.brief, &job.text_file].into_iter().flatten() {
                dependency(&root, base, path)?;
            }
            for reference in &job.references {
                dependency(&root, base, reference.path())?;
            }
            if let Some(path) = &job.character {
                let path = dependency(&root, base, path)?;
                contained_character(&root, &path)?;
            }
            if let Some(path) = &job.spec {
                let path = dependency(&root, base, path)?;
                contained_infographic(&root, &path)?;
            }
            for turn in &job.turns {
                for sample in &turn.voice_samples {
                    contained_sample(&root, base, sample)?;
                }
                dependency(&root, base, &turn.text_file)?;
                if let Some(path) = &turn.character {
                    let path = dependency(&root, base, path)?;
                    contained_character(&root, &path)?;
                }
            }
            if let Some(path) = &job.output {
                authoring_path(path)?;
            }
            validation::resolve(&job, base, config)?;
        }
        _ => unreachable!(),
    }
    Ok(manifest)
}

fn authoring_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(Error::input("Planner emitted a path outside its bundle"));
    }
    Ok(())
}
fn dependency(root: &Path, base: &Path, path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Err(Error::input(
            "Planner dependency must be relative and inside its bundle",
        ));
    }
    let resolved = relative(base, path)?;
    if !resolved.starts_with(root) {
        return Err(Error::input("Planner dependency escapes its bundle"));
    }
    Ok(resolved)
}
fn contained_character(root: &Path, manifest: &Path) -> Result<()> {
    let character: Character = yaml(manifest)?;
    let base = manifest.parent().unwrap();
    dependency(root, base, &character.persona)?;
    if let Some(profile) = &character.visual {
        dependency(root, base, &profile.description)?;
        for path in &profile.references {
            dependency(root, base, path)?;
        }
    }
    if let Some(profile) = &character.voice {
        dependency(root, base, &profile.description)?;
        for path in &profile.references {
            dependency(root, base, path)?;
        }
        for sample in &profile.samples {
            contained_sample(root, base, sample)?;
        }
    }
    Ok(())
}
fn contained_sample(root: &Path, base: &Path, sample: &VoiceSample) -> Result<()> {
    dependency(root, base, &sample.audio)?;
    dependency(root, base, &sample.transcript)?;
    let path = dependency(root, base, &sample.consent)?;
    let consent: VoiceConsent = yaml(&path)?;
    dependency(root, path.parent().unwrap(), &consent.evidence)?;
    Ok(())
}
fn contained_infographic(root: &Path, manifest: &Path) -> Result<()> {
    let spec: Infographic = yaml(manifest)?;
    let base = manifest.parent().unwrap();
    dependency(root, base, &spec.visual_brief)?;
    for reference in &spec.references {
        dependency(root, base, &reference.path)?;
    }
    Ok(())
}
