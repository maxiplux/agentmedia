use crate::{
    assets,
    contracts::*,
    error::{Error, Result},
    storage::*,
    validation,
};
use fs4::fs_std::FileExt;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Keep historical manifest paths, receipts and reference hashes intact.
/// New projects use framechord/; existing mediaagent/ projects stay in place.
pub fn directory(root: &Path) -> PathBuf {
    if root.join("mediaagent").is_dir() {
        root.join("mediaagent")
    } else {
        root.join("framechord")
    }
}

pub fn check_layout(root: &Path) -> Result<()> {
    if root.join("mediaagent").is_dir() && root.join("framechord").is_dir() {
        return Err(Error::new(
            "project_layout_conflict",
            "This project contains both framechord/ and legacy mediaagent/ directories. Keep the intended project directory at this root and move the other to a separate project before continuing. No project was selected or merged automatically.",
            2,
        ));
    }
    Ok(())
}

pub fn config(root: &Path) -> Result<Config> {
    check_layout(root)?;
    let path = crate::project::directory(root).join("config.yaml");
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
/// Detect the executable/configuration name collision before any setup files are written.
pub fn check_init_destination(root: &Path) -> Result<()> {
    let root = absolute(root)?;
    check_layout(&root)?;
    for path in root
        .ancestors()
        .chain(std::iter::once(crate::project::directory(&root).as_path()))
    {
        if fs::symlink_metadata(path).is_ok() && !path.is_dir() {
            let mut error = Error::new(
                "init_path_conflict",
                format!(
                    "Cannot initialize this project: {} is a file, but setup needs a directory. The framechord executable and the framechord/ configuration folder cannot share a path. Choose a project folder, for example: framechord init ./my-project. The existing file has not been changed.",
                    path.display()
                ),
                2,
            );
            error.field = Some("path".into());
            error.corrective_argv = vec!["framechord".into(), "init".into(), "./my-project".into()];
            return Err(error);
        }
    }
    Ok(())
}
pub fn init_with_config(
    root: &Path,
    agents: &[String],
    configured: Option<&Config>,
) -> Result<serde_json::Value> {
    // Reject invalid destinations/input without creating the setup lock directory.
    check_agents(agents)?;
    check_init_destination(root)?;
    if let Some(configured) = configured {
        validation::config(configured)?;
    }
    crate::credentials::render_missing(root, &Default::default())?;
    let guard = lock(&crate::project::directory(root).join("setup.lock"))?;
    guard.lock_exclusive()?;
    init_locked(root, agents, configured)
}
/// Caller holds setup.lock; re-read dotenv and configuration after acquiring it.
pub(crate) fn init_locked(
    root: &Path,
    agents: &[String],
    configured: Option<&Config>,
) -> Result<serde_json::Value> {
    check_agents(agents)?;
    check_init_destination(root)?;
    if let Some(configured) = configured {
        validation::config(configured)?;
    }
    let existing_path = crate::project::directory(root).join("config.yaml");
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
    // Check dotenv syntax before creating configuration or installing skills.
    let env_content = crate::credentials::render_missing(root, &Default::default())?;
    fs::create_dir_all(root)?;
    let root = fs::canonicalize(root)?;
    let cfg =
        serde_saphyr::to_string(&initial).map_err(|_| Error::input("Cannot serialize config"))?;
    let mut existing = vec![];
    if !create(
        &crate::project::directory(&root).join("config.yaml"),
        cfg.as_bytes(),
    )? {
        existing.push(directory(&root).join("config.yaml").display().to_string());
    }
    create(
        &root.join(".env.example"),
        crate::credentials::ENV_TEMPLATE.as_bytes(),
    )?;
    if let Some(content) = env_content {
        atomic(&root.join(".env"), content.as_bytes(), true)?;
    }
    create(
        &crate::project::directory(&root).join(".gitignore"),
        b"runs/\ncache/\nblobs/\nsetup.lock\n",
    )?;
    // Preserve existing ignore rules; add only missing credential/artifact protection.
    let ignore = root.join(".gitignore");
    let original = fs::read_to_string(&ignore).unwrap_or_default();
    let mut updated = original.clone();
    for rule in [
        ".env",
        ".env.*",
        "!.env.example",
        "/framechord/runs/",
        "/framechord/cache/",
        "/framechord/blobs/",
        "/framechord/setup.lock",
        "/mediaagent/runs/",
        "/mediaagent/cache/",
        "/mediaagent/blobs/",
        "/mediaagent/setup.lock",
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
    existing.extend(assets::templates(
        &crate::project::directory(&root).join("templates"),
    )?);
    for dir in ["characters", "jobs", "runs"] {
        fs::create_dir_all(crate::project::directory(&root).join(dir))?;
    }
    if !crate::project::directory(&root)
        .join("characters/persona")
        .exists()
    {
        add_persona(&root, "persona")?;
    }
    existing.extend(install(&root, agents)?);
    let config = config(&root)?;
    let configured_media = config.defaults.keys().collect::<Vec<_>>();
    Ok(
        serde_json::json!({"project":root,"data_directory":directory(&root),"legacy_layout":directory(&root).file_name().is_some_and(|name|name=="mediaagent"),"credentials_file":root.join(".env"),"credential_variables":crate::credentials::PROVIDERS.iter().map(|p|p.key()).collect::<Vec<_>>(),"existing_preserved":existing,"generation":false,"configured_media":configured_media,"configuration":config,"setup_required":configured_media.is_empty(),"next_steps":if configured_media.is_empty(){vec!["framechord config wizard","framechord config setup --file config.yaml"]}else{vec![]}}),
    )
}
/// Documented skill discovery locations per coding agent, verified against each
/// vendor's documentation on 2026-10-15. The first project/user directory is the
/// native install location; later ones are compatibility paths the agent also reads.
pub struct AgentTarget {
    pub name: &'static str,
    pub label: &'static str,
    pub project: &'static [&'static str],
    pub user: &'static [&'static str],
    pub source: &'static str,
}
pub const SKILL_FOLDER: &str = "framechord";
pub const DEFAULT_AGENTS: &[&str] = &["codex", "claude"];
pub const AGENT_TARGETS: &[AgentTarget] = &[
    AgentTarget {
        name: "codex",
        label: "Codex",
        project: &[".agents/skills"],
        user: &[".agents/skills"],
        source: "https://developers.openai.com/codex/skills",
    },
    AgentTarget {
        name: "claude",
        label: "Claude Code",
        project: &[".claude/skills"],
        user: &[".claude/skills"],
        source: "https://code.claude.com/docs/en/skills",
    },
    AgentTarget {
        name: "cursor",
        label: "Cursor",
        project: &[
            ".cursor/skills",
            ".agents/skills",
            ".claude/skills",
            ".codex/skills",
        ],
        user: &[
            ".cursor/skills",
            ".agents/skills",
            ".claude/skills",
            ".codex/skills",
        ],
        source: "https://cursor.com/docs/skills",
    },
    AgentTarget {
        name: "gemini",
        label: "Gemini CLI",
        project: &[".gemini/skills", ".agents/skills"],
        user: &[".gemini/skills"],
        source: "https://geminicli.com/docs/cli/tutorials/skills-getting-started/",
    },
    AgentTarget {
        name: "opencode",
        label: "OpenCode",
        project: &[".opencode/skills", ".claude/skills", ".agents/skills"],
        user: &[
            ".config/opencode/skills",
            ".claude/skills",
            ".agents/skills",
        ],
        source: "https://opencode.ai/docs/skills/",
    },
    AgentTarget {
        name: "copilot",
        label: "GitHub Copilot",
        project: &[".github/skills", ".claude/skills", ".agents/skills"],
        user: &[".copilot/skills", ".claude/skills", ".agents/skills"],
        source: "https://docs.github.com/en/copilot/concepts/agents/about-agent-skills",
    },
];
pub fn agent_target(name: &str) -> Option<&'static AgentTarget> {
    AGENT_TARGETS.iter().find(|target| target.name == name)
}
fn agent_target_error() -> Error {
    Error::input(
        "Agent target must be one of codex, claude, cursor, gemini, opencode, copilot, or none; use skills export DIRECTORY for any other agent",
    )
}
/// Trims and de-duplicates targets while preserving order. `none` is only valid alone.
pub fn normalize_agents(agents: &[String]) -> Result<Vec<&'static AgentTarget>> {
    let names: Vec<&str> = agents
        .iter()
        .map(|agent| agent.trim())
        .filter(|agent| !agent.is_empty())
        .collect();
    if names.contains(&"none") {
        if names.len() > 1 {
            return Err(Error::input(
                "Agent target none must be given alone: pass --agents none to install no skill, or list only the agents to install",
            ));
        }
        return Ok(vec![]);
    }
    if names.is_empty() && !agents.is_empty() {
        return Err(agent_target_error());
    }
    let mut selected: Vec<&'static AgentTarget> = vec![];
    for name in names {
        let target = agent_target(name).ok_or_else(agent_target_error)?;
        if !selected.iter().any(|existing| existing.name == target.name) {
            selected.push(target);
        }
    }
    Ok(selected)
}
pub fn check_agents(agents: &[String]) -> Result<()> {
    normalize_agents(agents).map(|_| ())
}
/// Joins a documented skill directory such as `.config/opencode/skills` onto `root`
/// one component at a time and appends the skill folder, so the installer and the
/// status report both produce a path in the platform's own separator throughout
/// (a single `join` would keep the literal `/` on Windows).
pub fn skill_dir(root: &Path, dir: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    path.extend(dir.split('/').filter(|component| !component.is_empty()));
    path.push(SKILL_FOLDER);
    path
}
pub fn native_skill_dir(root: &Path, target: &AgentTarget, user_scope: bool) -> PathBuf {
    let dirs = if user_scope {
        target.user
    } else {
        target.project
    };
    skill_dir(root, dirs[0])
}
pub fn install(root: &Path, agents: &[String]) -> Result<Vec<String>> {
    install_with(root, agents, false)
}
pub fn install_with(root: &Path, agents: &[String], overwrite: bool) -> Result<Vec<String>> {
    install_scoped(root, agents, overwrite, false)
}
/// Installs the bundled skill into each selected agent's native directory under
/// `root` (a project, or the home directory when `user_scope` is set).
pub fn install_scoped(
    root: &Path,
    agents: &[String],
    overwrite: bool,
    user_scope: bool,
) -> Result<Vec<String>> {
    let mut existing = vec![];
    for target in normalize_agents(agents)? {
        existing.extend(assets::skill_with(
            &native_skill_dir(root, target, user_scope),
            overwrite,
        )?);
    }
    Ok(existing)
}
/// Per-agent discoverability of the project skill: `skills` says whether a FrameChord
/// SKILL.md exists in any directory that agent documents; `skill_targets` carries the
/// native path, the directories through which the skill is actually found, and
/// `duplicate_discovery` when more than one of them holds a copy.
pub fn skills_report(root: &Path) -> serde_json::Value {
    let skill_at = |dir: &str| skill_dir(root, dir);
    let installed = |dir: &str| skill_at(dir).join("SKILL.md").is_file();
    let mut skills = serde_json::Map::new();
    let mut targets = vec![];
    let mut shared: std::collections::BTreeMap<&str, Vec<&str>> = Default::default();
    for target in AGENT_TARGETS {
        let discovered: Vec<PathBuf> = target
            .project
            .iter()
            .filter(|dir| installed(dir))
            .map(|dir| skill_at(dir))
            .collect();
        skills.insert(target.name.to_owned(), (!discovered.is_empty()).into());
        // More than one discovered copy means the agent sees two skills with the same
        // name (native plus shared directory); docs/INTEGRATION.md asks users to keep one.
        targets.push(serde_json::json!({"agent":target.name,"label":target.label,"native_path":skill_at(target.project[0]),"native_installed":installed(target.project[0]),"discovered_via":discovered,"duplicate_discovery":discovered.len()>1,"source":target.source}));
        for dir in target.project {
            shared.entry(dir).or_default().push(target.name);
        }
    }
    shared.retain(|_, agents| agents.len() > 1);
    serde_json::json!({"skills":skills,"skill_targets":targets,"skills_shared_by":shared})
}
pub fn add_persona(root: &Path, name: &str) -> Result<PathBuf> {
    id(name)?;
    let configured = config(root)?;
    let dir = crate::project::directory(root)
        .join("characters")
        .join(name);
    if dir.exists() {
        return Err(Error::input(
            "Character ID already exists; inspect or import with a different ID",
        ));
    }
    fs::create_dir_all(dir.parent().unwrap())?;
    fs::create_dir(&dir).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            Error::input("Character ID already exists; choose another ID")
        } else {
            Error::io(error)
        }
    })?;
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
    Ok(crate::project::directory(root)
        .join("characters")
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
        &crate::project::directory(root)
            .join("characters")
            .join(name),
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
