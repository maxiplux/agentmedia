use crate::{
    assets,
    contracts::*,
    error::{Error, Result},
    storage::*,
    validation,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn config(root: &Path) -> Result<Config> {
    let path = root.join("mediaagent/config.yaml");
    let cfg = if path.exists() {
        yaml(&path)?
    } else {
        Config::default()
    };
    validation::config(&cfg)?;
    Ok(cfg)
}
pub fn init(root: &Path, agents: &[String]) -> Result<serde_json::Value> {
    init_with_config(root, agents, None)
}
pub fn init_with_config(
    root: &Path,
    agents: &[String],
    configured: Option<&Config>,
) -> Result<serde_json::Value> {
    check_agents(agents)?;
    if let Some(configured) = configured {
        validation::config(configured)?;
    }
    let existing_path = root.join("mediaagent/config.yaml");
    if existing_path.exists()
        && let Some(configured) = configured
    {
        let existing: Config = yaml(&existing_path)?;
        if serde_json::to_value(&existing)? != serde_json::to_value(configured)? {
            return Err(Error::input(
                "Existing configuration differs; init preserves it. Use config setup --file or config wizard for an explicit update",
            ));
        }
    }
    let mut initial = configured.cloned().unwrap_or_default();
    if configured.is_none() {
        initial.consistency.require_known_identity_model = true;
    }
    validation::config(&initial)?;
    fs::create_dir_all(root)?;
    let root = fs::canonicalize(root)?;
    let cfg =
        serde_saphyr::to_string(&initial).map_err(|_| Error::input("Cannot serialize config"))?;
    let mut existing = vec![];
    if !create(&root.join("mediaagent/config.yaml"), cfg.as_bytes())? {
        existing.push("mediaagent/config.yaml".into());
    }
    create(&root.join(".env.example"),b"OPENAI_API_KEY=\nGEMINI_API_KEY=\nFISH_API_KEY=\nOPENROUTER_API_KEY=\nELEVENLABS_API_KEY=\n")?;
    create(&root.join(".env"), b"")?;
    create(
        &root.join("mediaagent/.gitignore"),
        b"runs/\ncache/\nblobs/\n",
    )?;
    // Preserve existing ignore rules; add only missing credential/artifact protection.
    let ignore = root.join(".gitignore");
    let original = fs::read_to_string(&ignore).unwrap_or_default();
    let mut updated = original.clone();
    for rule in [
        ".env",
        ".env.*",
        "!.env.example",
        "/mediaagent/runs/",
        "/mediaagent/cache/",
        "/mediaagent/blobs/",
    ] {
        if !original.lines().any(|l| l == rule) {
            if !updated.ends_with('\n') && !updated.is_empty() {
                updated.push('\n');
            }
            updated.push_str(rule);
            updated.push('\n');
        }
    }
    if updated != original {
        atomic(&ignore, updated.as_bytes(), true)?;
    }
    existing.extend(assets::templates(&root.join("mediaagent/templates"))?);
    for dir in ["characters", "jobs", "runs"] {
        fs::create_dir_all(root.join("mediaagent").join(dir))?;
    }
    if !root.join("mediaagent/characters/persona").exists() {
        add_persona(&root, "persona")?;
    }
    existing.extend(install(&root, agents)?);
    let config = config(&root)?;
    let configured_media = config.defaults.keys().collect::<Vec<_>>();
    Ok(
        serde_json::json!({"project":root,"existing_preserved":existing,"generation":false,"configured_media":configured_media,"configuration":config,"setup_required":configured_media.is_empty(),"next_steps":if configured_media.is_empty(){vec!["mediaagent config wizard","mediaagent config setup --file config.yaml"]}else{vec![]}}),
    )
}
pub fn install(root: &Path, agents: &[String]) -> Result<Vec<String>> {
    install_with(root, agents, false)
}
pub fn install_with(root: &Path, agents: &[String], overwrite: bool) -> Result<Vec<String>> {
    check_agents(agents)?;
    let mut existing = vec![];
    for agent in agents {
        let dir = match agent.as_str() {
            "codex" => ".agents/skills/mediaagent",
            "claude" => ".claude/skills/mediaagent",
            _ => {
                return Err(Error::input(
                    "Agent target must be codex or claude; use skills export for any other agent",
                ));
            }
        };
        existing.extend(assets::skill_with(&root.join(dir), overwrite)?);
    }
    Ok(existing)
}
fn check_agents(agents: &[String]) -> Result<()> {
    if agents.iter().any(|a| a != "codex" && a != "claude") {
        return Err(Error::input(
            "Agent target must be codex or claude; use skills export for other agents",
        ));
    }
    Ok(())
}
pub fn add_persona(root: &Path, name: &str) -> Result<PathBuf> {
    id(name)?;
    let configured = config(root)?;
    let dir = root.join("mediaagent/characters").join(name);
    if dir.exists() {
        return Err(Error::input(
            "Character ID already exists; inspect or import with a different ID",
        ));
    }
    fs::create_dir_all(dir.join("references/images"))?;
    fs::create_dir_all(dir.join("references/audio"))?;
    for filename in ["persona.md", "visual.md", "voice.md"] {
        let data = assets::TEMPLATES
            .iter()
            .find(|(n, _)| *n == filename)
            .unwrap()
            .1;
        create(&dir.join(filename), data.as_bytes())?;
    }
    save(
        &dir.join("character.yaml"),
        &Character {
            schema_version: 1,
            id: name.into(),
            persona: "persona.md".into(),
            visual: configured
                .defaults
                .get(&Media::Image)
                .map(|selection| Visual {
                    description: "visual.md".into(),
                    selection: selection.clone(),
                    settings: configured
                        .default_settings
                        .get(&Media::Image)
                        .cloned()
                        .unwrap_or_default(),
                    origins: Default::default(),
                    references: vec![],
                }),
            voice: configured
                .defaults
                .get(&Media::Speech)
                .map(|selection| Voice {
                    description: "voice.md".into(),
                    selection: selection.clone(),
                    binding: configured
                        .default_settings
                        .get(&Media::Speech)
                        .and_then(|s| s.voice.clone()),
                    references: vec![],
                    samples: vec![],
                }),
        },
    )?;
    Ok(dir.join("character.yaml"))
}
pub fn character_path(root: &Path, name: &str) -> Result<PathBuf> {
    id(name)?;
    Ok(root
        .join("mediaagent/characters")
        .join(name)
        .join("character.yaml"))
}
pub fn bundle(source: &Path, destination: &Path, new_id: Option<&str>) -> Result<PathBuf> {
    let mut c = validation::character(source)?;
    if let Some(name) = new_id {
        id(name)?;
        c.id = name.into();
    }
    if destination.exists() {
        return Err(Error::input("Bundle/character destination already exists"));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| Error::input("Bundle needs parent directory"))?;
    fs::create_dir_all(parent)?;
    let stage = tempfile::tempdir_in(parent)?;
    let base = source.parent().unwrap();
    fn copy(base: &Path, src: &Path, stage: &Path, name: &Path) -> Result<PathBuf> {
        let bytes = fs::read(relative(base, src)?)?;
        atomic(&stage.join(name), &bytes, false)?;
        Ok(name.to_owned())
    }
    c.persona = copy(base, &c.persona, stage.path(), Path::new("persona.md"))?;
    if let Some(v) = &mut c.visual {
        v.description = copy(base, &v.description, stage.path(), Path::new("visual.md"))?;
        for (i, p) in v.references.iter_mut().enumerate() {
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("png");
            *p = copy(
                base,
                p,
                stage.path(),
                Path::new(&format!("references/images/{i}.{ext}")),
            )?;
        }
    }
    if let Some(v) = &mut c.voice {
        v.description = copy(base, &v.description, stage.path(), Path::new("voice.md"))?;
        for (i, p) in v.references.iter_mut().enumerate() {
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("wav");
            *p = copy(
                base,
                p,
                stage.path(),
                Path::new(&format!("references/audio/{i}.{ext}")),
            )?;
        }
        for (i, sample) in v.samples.iter_mut().enumerate() {
            let consent_path = relative(base, &sample.consent)?;
            let mut consent: VoiceConsent = yaml(&consent_path)?;
            let ext = sample
                .audio
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("wav");
            sample.audio = copy(
                base,
                &sample.audio,
                stage.path(),
                Path::new(&format!("references/audio/sample-{i}.{ext}")),
            )?;
            sample.transcript = copy(
                base,
                &sample.transcript,
                stage.path(),
                Path::new(&format!("references/audio/sample-{i}-transcript.md")),
            )?;
            let evidence_name = format!("sample-{i}-evidence.md");
            copy(
                consent_path.parent().unwrap(),
                &consent.evidence,
                stage.path(),
                Path::new(&format!("references/audio/{evidence_name}")),
            )?;
            consent.evidence = evidence_name.into();
            sample.consent = format!("references/audio/sample-{i}-consent.yaml").into();
            save(&stage.path().join(&sample.consent), &consent)?;
        }
    }
    save(&stage.path().join("character.yaml"), &c)?;
    validation::character(&stage.path().join("character.yaml"))?;
    fs::rename(stage.path(), destination)?;
    Ok(destination.join("character.yaml"))
}
pub fn import(root: &Path, source: &Path, new_id: Option<&str>) -> Result<PathBuf> {
    let source = source.join("character.yaml");
    let c = validation::character(&source)?;
    // Imported manifests must be portable and contained, even when symbolic links are present.
    let base = fs::canonicalize(source.parent().unwrap())?;
    let mut deps = vec![&c.persona];
    if let Some(v) = &c.visual {
        deps.push(&v.description);
        deps.extend(v.references.iter());
    }
    if let Some(v) = &c.voice {
        deps.push(&v.description);
        deps.extend(v.references.iter());
        for sample in &v.samples {
            deps.extend([&sample.audio, &sample.transcript, &sample.consent]);
            let cp = relative(&base, &sample.consent)?;
            let consent: VoiceConsent = yaml(&cp)?;
            if consent.evidence.is_absolute()
                || !relative(cp.parent().unwrap(), &consent.evidence)?.starts_with(&base)
            {
                return Err(Error::input("Consent evidence escapes the portable bundle"));
            }
        }
    }
    for p in deps {
        if p.is_absolute() || !relative(&base, p)?.starts_with(&base) {
            return Err(Error::input(
                "Import dependency escapes the portable bundle",
            ));
        }
    }
    let name = new_id.unwrap_or(&c.id);
    id(name)?;
    bundle(
        &source,
        &root.join("mediaagent/characters").join(name),
        new_id,
    )
}
pub fn accept(path: &Path, reference_path: &Path, kind: &str) -> Result<PathBuf> {
    accept_origin(path, reference_path, kind, None)
}
pub fn accept_project(
    root: &Path,
    path: &Path,
    reference_path: &Path,
    kind: &str,
) -> Result<PathBuf> {
    let reference = validation::reference(Path::new("."), reference_path)?;
    let index = crate::provenance::Index::load(root)?;
    let origin = index.origin(Some(reference_path), &reference.sha256, kind == "image")?;
    accept_origin(path, reference_path, kind, origin)
}
fn accept_origin(
    path: &Path,
    reference_path: &Path,
    kind: &str,
    origin: Option<AssetOrigin>,
) -> Result<PathBuf> {
    let file = lock(&path.with_extension("lock"))?;
    fs4::fs_std::FileExt::lock_exclusive(&file)?;
    let mut c = validation::character(path)?;
    let base = path.parent().unwrap();
    let r = validation::reference(Path::new("."), reference_path)?;
    let sub = match kind {
        "image" if r.mime.starts_with("image/") => "images",
        "audio" if r.mime.starts_with("audio/") => "audio",
        _ => return Err(Error::input("Reference kind does not match the file")),
    };
    let dest = PathBuf::from(format!("references/{sub}/{}-{}", &r.sha256[..12], r.name));
    if kind == "image" {
        let visual = c
            .visual
            .as_mut()
            .ok_or_else(|| Error::input("Enable a visual profile before accepting images"))?;
        let origin = origin.unwrap_or_else(|| AssetOrigin {
            selection: visual.selection.clone(),
            settings: visual.settings.clone(),
            locale: None,
            sha256: r.sha256.clone(),
            artifact: None,
        });
        if origin.selection != visual.selection {
            return Err(Error::input(
                "Character model differs from the original asset; configure the character to use the original provider/model before accepting it",
            ));
        }
        if visual.settings == Settings::default() {
            visual.settings = origin.settings.clone();
        }
        visual.origins.insert(r.sha256.clone(), origin);
        if !visual.references.contains(&dest) {
            visual.references.push(dest.clone());
        }
    } else {
        c.voice
            .as_mut()
            .ok_or_else(|| {
                Error::input("Enable and configure a voice profile before accepting recordings")
            })?
            .references
            .push(dest.clone());
    }
    create(&base.join(&dest), &fs::read(reference_path)?)?;
    save(path, &c)?;
    Ok(base.join(dest))
}

/// Explicitly accept a recording with its exact transcript and authorization evidence.
pub fn accept_sample(path: &Path, sample: &VoiceSample) -> Result<VoiceSample> {
    let file = lock(&path.with_extension("lock"))?;
    fs4::fs_std::FileExt::lock_exclusive(&file)?;
    let mut c = validation::character(path)?;
    let reference =
        validation::voice_sample(Path::new("."), sample, ConsentPurpose::ReferenceSynthesis)?;
    let voice = c
        .voice
        .as_mut()
        .ok_or_else(|| Error::input("Enable a voice profile before accepting a sample"))?;
    if voice.selection.provider != Provider::Fish && voice.binding.is_none() {
        return Err(Error::input(
            "Reference synthesis is supported for Fish; configure a saved binding for other providers before accepting review evidence",
        ));
    }
    let fingerprint = hash(&serde_json::to_vec(&(
        &reference.sha256,
        &reference.transcript,
        &reference.consent_hash,
    ))?);
    let folder = PathBuf::from(format!("references/audio/sample-{fingerprint}"));
    let base = path.parent().unwrap();
    let dest = VoiceSample {
        audio: folder.join(&reference.name),
        transcript: folder.join("transcript.md"),
        consent: folder.join("consent.yaml"),
    };
    if voice.samples.contains(&dest) {
        return Ok(dest);
    }
    let source_consent = relative(Path::new("."), &sample.consent)?;
    let mut consent: VoiceConsent = yaml(&source_consent)?;
    let evidence = text(source_consent.parent().unwrap(), &consent.evidence)?;
    consent.evidence = "evidence.md".into();
    let destination = base.join(&folder);
    if !destination.exists() {
        fs::create_dir_all(destination.parent().unwrap())?;
        let stage = tempfile::tempdir_in(destination.parent().unwrap())?;
        atomic(
            &stage.path().join(&reference.name),
            &fs::read(&sample.audio)?,
            false,
        )?;
        atomic(
            &stage.path().join("transcript.md"),
            reference.transcript.as_ref().unwrap().as_bytes(),
            false,
        )?;
        atomic(
            &stage.path().join("evidence.md"),
            evidence.as_bytes(),
            false,
        )?;
        save(&stage.path().join("consent.yaml"), &consent)?;
        fs::rename(stage.path(), &destination)?;
    }
    let copied = validation::voice_sample(base, &dest, ConsentPurpose::ReferenceSynthesis)?;
    if copied.sha256 != reference.sha256
        || copied.transcript != reference.transcript
        || copied.consent_hash != reference.consent_hash
    {
        return Err(Error::input(
            "Accepted sample content does not match its source",
        ));
    }
    voice.samples.push(dest.clone());
    save(path, &c)?;
    Ok(dest)
}
