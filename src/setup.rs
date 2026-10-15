//! Explicit local configuration and terminal-only authoring assistance.
use crate::{
    catalog,
    contracts::*,
    credentials,
    error::{Error, Result},
    project,
    storage::*,
    validation,
};
use fs4::fs_std::FileExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HostLimits {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub concurrency: usize,
    pub per_provider: usize,
}
impl Default for HostLimits {
    fn default() -> Self {
        Self {
            schema_version: 1,
            concurrency: 4,
            per_provider: 2,
        }
    }
}
pub fn validate_host(limits: &HostLimits) -> Result<()> {
    version(limits.schema_version)?;
    if limits.concurrency == 0
        || limits.concurrency > 32
        || limits.per_provider == 0
        || limits.per_provider > 16
        || limits.per_provider > limits.concurrency
    {
        return Err(Error::input(
            "Host limits need 1–32 global slots and 1–16 provider slots, with provider slots at most global slots",
        ));
    }
    Ok(())
}
pub fn host_path() -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or_else(|| Error::input("Cannot locate user home for framechord host settings"))?;
    host_path_in(Path::new(&home))
}
fn host_path_in(home: &Path) -> Result<PathBuf> {
    let current = home.join(".config/framechord/host.yaml");
    let legacy = home.join(".config/mediaagent/host.yaml");
    if legacy.exists() {
        if current.exists() {
            return Err(Error::input(
                "Both FrameChord and MediaAgent host limits exist. Stop workers and reconcile these files explicitly before continuing; neither limit was selected automatically",
            ));
        }
        Ok(legacy)
    } else {
        Ok(current)
    }
}
pub fn host() -> Result<HostLimits> {
    let path = host_path()?;
    let limits = if path.exists() {
        yaml(&path)?
    } else {
        HostLimits::default()
    };
    validate_host(&limits)?;
    Ok(limits)
}
pub fn save_host(path: &Path, limits: &HostLimits) -> Result<()> {
    validate_host(limits)?;
    save(path, limits)
}

pub fn recommended() -> Config {
    preset(Provider::Google)
}
pub fn openai() -> Config {
    preset(Provider::Openai)
}
fn preset(provider: Provider) -> Config {
    let mut config = Config::default();
    config.consistency.require_known_identity_model = true;
    let google = provider == Provider::Google;
    for (media, model) in [
        (
            Media::Image,
            if google {
                "gemini-nano-banana-2.1"
            } else {
                "gpt-image-2.5-flare"
            },
        ),
        (
            Media::Infographic,
            if google {
                "gemini-nano-banana-2.1"
            } else {
                "gpt-image-2.5-flare"
            },
        ),
        (
            Media::Speech,
            if google {
                "gemini-3.8-flash-tts"
            } else {
                "gpt-4o-mini-tts"
            },
        ),
        (
            Media::Podcast,
            if google {
                "gemini-3.8-flash-tts"
            } else {
                "gpt-4o-mini-tts"
            },
        ),
        (
            Media::Text,
            if google {
                "gemini-3.8-flash"
            } else {
                "gpt-6-luna"
            },
        ),
    ] {
        config.defaults.insert(
            media,
            Selection {
                provider,
                model: model.into(),
            },
        );
        config
            .default_settings
            .insert(media, defaults(media, provider));
    }
    if google {
        config.defaults.insert(
            Media::Music,
            Selection {
                provider,
                model: "lyria-3.5".into(),
            },
        );
        config
            .default_settings
            .insert(Media::Music, defaults(Media::Music, provider));
    }
    config.default_settings.remove(&Media::Text);
    config
}
fn defaults(media: Media, provider: Provider) -> Settings {
    let mut settings = Settings::default();
    match catalog::target(media) {
        Media::Image => {
            settings.format = Some("png".into());
            if provider == Provider::Openai {
                settings.size = Some(
                    if media == Media::Infographic {
                        "1024x1536"
                    } else {
                        "1024x1024"
                    }
                    .into(),
                );
                settings.image_quality = Some("high".into());
            }
        }
        Media::Speech => {
            settings.format = Some("wav".into());
            settings.voice = match provider {
                Provider::Google => Some("Kore".into()),
                Provider::Openai => Some("coral".into()),
                _ => None,
            };
        }
        Media::Music | Media::Sfx => {
            settings.format = Some("mp3".into());
            settings.duration_seconds = Some(if media == Media::Music {
                if provider == Provider::Google { 60 } else { 10 }
            } else {
                3
            });
            if media == Media::Music {
                settings.instrumental = Some(true);
            }
        }
        _ => {}
    }
    settings
}
fn readiness(
    root: &Path,
    config: &Config,
    pending: &std::collections::BTreeMap<Provider, String>,
) -> serde_json::Value {
    let mut issues = Vec::new();
    if let Err(error) = validation::config(config) {
        issues.push(error.to_string());
    }
    let mut rows = Vec::new();
    for (media, selection) in &config.defaults {
        let mut missing = Vec::new();
        if let Err(error) = catalog::require_current(selection) {
            missing.push(error.to_string());
        }
        if !pending.contains_key(&selection.provider)
            && credentials::resolve(selection.provider, root).is_err()
        {
            missing.push(format!(
                "Set {} in the environment or project .env",
                selection.provider.key()
            ));
        }
        if catalog::target(*media) == Media::Speech
            && config
                .default_settings
                .get(media)
                .and_then(|s| s.voice.as_ref())
                .is_none()
        {
            missing.push(format!("Choose a provider voice ID for {media:?}"));
        }
        issues.extend(missing.clone());
        rows.push(serde_json::json!({"media":media,"selection":selection,"ready":missing.is_empty(),"pending_steps":missing}));
    }
    issues.sort();
    issues.dedup();
    serde_json::json!({"setup_complete":issues.is_empty(),"setup_required":!issues.is_empty(),"pending_steps":issues,"features":rows,"enabled_features":config.defaults.len(),"account_access_verified":false,"generation":false,"catalog_verified_at":catalog::VERIFIED_AT})
}
pub fn status(root: &Path) -> Result<serde_json::Value> {
    let mut result = readiness(root, &project::config(root)?, &Default::default());
    if !crate::project::directory(root).join("config.yaml").exists() {
        result["setup_complete"] = false.into();
        result["setup_required"] = true.into();
        result["pending_steps"]
            .as_array_mut()
            .unwrap()
            .push("Initialize this project first with framechord init".into());
    }
    for (key, value) in project::skills_report(root).as_object().unwrap() {
        result[key] = value.clone();
    }
    Ok(result)
}

/// Secrets deliberately have no Debug/Serialize implementation and stay out of configuration.
pub struct Session {
    pub config: Config,
    pub agents: Vec<String>,
    pending_keys: std::collections::BTreeMap<Provider, String>,
    personas: Vec<String>,
}
impl Session {
    pub fn new(config: Config, agents: Vec<String>) -> Self {
        Self {
            config,
            agents,
            pending_keys: Default::default(),
            personas: vec![],
        }
    }
    pub fn report(&self, root: &Path) -> serde_json::Value {
        readiness(root, &self.config, &self.pending_keys)
    }
    pub fn finishable(&self, root: &Path) -> bool {
        self.report(root)["setup_complete"] == true
    }
    pub fn save(&self, root: &Path) -> Result<serde_json::Value> {
        self.commit(root, None)
    }
    fn commit(
        &self,
        root: &Path,
        expected: Option<(&Option<String>, &Option<String>)>,
    ) -> Result<serde_json::Value> {
        validation::config(&self.config)?;
        project::check_agents(&self.agents)?;
        project::check_init_destination(root)?;
        let guard = lock(&crate::project::directory(root).join("setup.lock"))?;
        guard.lock_exclusive()?;
        if expected.is_some_and(|(config_hash, env_hash)| {
            &std::fs::read(crate::project::directory(root).join("config.yaml"))
                .ok()
                .map(|b| hash(&b))
                != config_hash
                || &std::fs::read(root.join(".env")).ok().map(|b| hash(&b)) != env_hash
        }) {
            return Err(Error::input(
                "Configuration or credentials changed during setup; restart to review the current files",
            ));
        }
        let credentials = credentials::render_missing(root, &self.pending_keys)?;
        for name in &self.personas {
            crate::storage::id(name)?;
            if crate::project::directory(root)
                .join("characters")
                .join(name)
                .exists()
            {
                return Err(Error::input(
                    "Character ID already exists; choose another ID",
                ));
            }
        }
        let existing = crate::project::directory(root).join("config.yaml").exists();
        let mut result = project::init_locked(
            root,
            &self.agents,
            if existing { None } else { Some(&self.config) },
        )?;
        if let Some(content) = credentials {
            atomic(&root.join(".env"), content.as_bytes(), true)?;
        }
        if existing
            && serde_json::to_value(project::config(root)?)? != serde_json::to_value(&self.config)?
        {
            save(
                &crate::project::directory(root).join("config.yaml"),
                &self.config,
            )?;
        }
        for name in &self.personas {
            project::add_persona(root, name)?;
        }
        let report = status(root)?;
        for (k, v) in report.as_object().unwrap() {
            result[k] = v.clone();
        }
        result["configuration"] = serde_json::to_value(&self.config)?;
        Ok(result)
    }
}
struct Ui {
    term: console::Term,
    theme: dialoguer::theme::ColorfulTheme,
    plain: bool,
    color: bool,
}
impl Ui {
    fn new(color: crate::cli::Color) -> Self {
        let enabled = match color {
            crate::cli::Color::Always => true,
            crate::cli::Color::Never => false,
            crate::cli::Color::Auto => {
                std::env::var_os("NO_COLOR").is_none()
                    && std::env::var("TERM").is_ok_and(|v| v != "dumb")
            }
        };
        console::set_colors_enabled_stderr(enabled);
        Self {
            term: console::Term::stderr(),
            theme: dialoguer::theme::ColorfulTheme {
                active_item_style: console::Style::new().for_stderr().cyan().bold(),
                active_item_prefix: console::style("❯".to_owned()).for_stderr().cyan(),
                ..Default::default()
            },
            plain: std::env::var("TERM").is_ok_and(|v| v == "dumb"),
            color: enabled,
        }
    }
    fn choose(&self, label: &str, items: &[String], default: usize) -> Result<Option<usize>> {
        if !self.plain {
            return dialoguer::FuzzySelect::with_theme(&self.theme)
                .with_prompt(label)
                .items(items)
                .default(default)
                .max_length(12)
                .interact_on_opt(&self.term)
                .map_err(|_| {
                    Error::new(
                        "setup_cancelled",
                        "Setup interrupted; unsaved choices were discarded",
                        130,
                    )
                });
        }
        loop {
            eprintln!("\n{label}");
            for (i, item) in items.iter().enumerate() {
                eprintln!("  {}. {item}", i + 1);
            }
            eprint!("Choice [{}], or b to go back: ", default + 1);
            io::stderr().flush()?;
            let mut line = String::new();
            if io::stdin().read_line(&mut line)? == 0 {
                return Err(Error::new(
                    "setup_cancelled",
                    "Input ended; unsaved choices were discarded",
                    130,
                ));
            }
            let line = line.trim();
            if line == "b" {
                return Ok(None);
            }
            if line.is_empty() {
                return Ok(Some(default));
            }
            if let Ok(n) = line.parse::<usize>()
                && n > 0
                && n <= items.len()
            {
                return Ok(Some(n - 1));
            }
            self.message("Enter a listed number", false);
        }
    }
    fn input(&self, label: &str, current: &str) -> Result<String> {
        dialoguer::Input::<String>::with_theme(&self.theme)
            .with_prompt(label)
            .with_initial_text(current)
            .allow_empty(true)
            .interact_text_on(&self.term)
            .map_err(|_| {
                Error::new(
                    "setup_cancelled",
                    "Setup interrupted; unsaved choices were discarded",
                    130,
                )
            })
    }
    fn message(&self, text: &str, ok: bool) {
        let style = if ok {
            console::Style::new().green()
        } else {
            console::Style::new().yellow()
        };
        eprintln!("{}", style.force_styling(self.color).apply_to(text));
    }
}
fn options(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}
pub fn init_destination(root: &Path, color: crate::cli::Color) -> Result<PathBuf> {
    let mut selected = absolute(root)?;
    let ui = Ui::new(color);
    loop {
        match project::check_init_destination(&selected) {
            Ok(()) => return Ok(selected),
            Err(error) if error.code == "init_path_conflict" => {
                if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
                    return Err(error);
                }
                ui.message(&error.message, false);
                if ui.choose(
                    "Choose where to save your project",
                    &options(&["Choose a project folder", "Cancel setup"]),
                    0,
                )? != Some(0)
                {
                    return Err(Error::new(
                        "setup_cancelled",
                        "Setup cancelled; no files changed",
                        130,
                    ));
                }
                let path = ui.input("Project folder (relative to the current directory)", "")?;
                if path.trim().is_empty() {
                    ui.message("Enter a project folder, for example ./my-project", false);
                    continue;
                }
                selected = absolute(Path::new(&path))?;
            }
            Err(error) => return Err(error),
        }
    }
}
const MEDIA: [Media; 7] = [
    Media::Image,
    Media::Infographic,
    Media::Speech,
    Media::Podcast,
    Media::Text,
    Media::Music,
    Media::Sfx,
];
fn feature(ui: &Ui, session: &mut Session, root: &Path) -> Result<()> {
    let items = MEDIA
        .iter()
        .map(|m| {
            format!(
                "{m:?}: {}",
                session
                    .config
                    .defaults
                    .get(m)
                    .map_or("disabled".into(), |s| format!(
                        "{:?} / {}",
                        s.provider, s.model
                    ))
            )
        })
        .collect::<Vec<_>>();
    let Some(i) = ui.choose("Choose a feature", &items, 0)? else {
        return Ok(());
    };
    let media = MEDIA[i];
    let Some(action) = ui.choose(
        "Model view",
        &options(&[
            "Recommended current models",
            "All current supported models",
            "Browse provider catalog (including unavailable models)",
            "Disable this feature",
        ]),
        0,
    )?
    else {
        return Ok(());
    };
    if action == 3 {
        session.config.defaults.remove(&media);
        session.config.default_settings.remove(&media);
        session.config.default_locales.remove(&media);
        return Ok(());
    }
    if action == 2 {
        let value = catalog::listing(
            root,
            &catalog::Filter {
                media: Some(media),
                all: true,
                ..Default::default()
            },
        );
        let items = value["models"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                format!(
                    "{}: {} [{}]",
                    m["provider"].as_str().unwrap_or(""),
                    m["model"].as_str().unwrap_or(""),
                    if m["current"] == true && m["supported"] == true {
                        "selectable"
                    } else {
                        "unavailable"
                    }
                )
            })
            .collect::<Vec<_>>();
        let Some(index) =
            ui.choose("Catalog — unavailable models cannot be selected", &items, 0)?
        else {
            return Ok(());
        };
        let row = &value["models"][index];
        if row["current"] != true || row["supported"] != true {
            ui.message("This model cannot generate. Use a current supported model; live discovery never grants execution support.",false);
            return Ok(());
        }
        let model: catalog::Model = serde_json::from_value(row.clone())?;
        set_model(session, media, model);
    } else {
        let mut rows = catalog::models()
            .into_iter()
            .filter(|m| {
                m.media == catalog::target(media)
                    && m.current
                    && m.supported
                    && (action == 1 || m.recommended)
            })
            .collect::<Vec<_>>();
        rows.sort_by_key(|m| (catalog::provider_preference(m.provider), !m.recommended));
        let items = rows
            .iter()
            .map(|m| format!("{:?} / {} — {}", m.provider, m.model, m.description))
            .collect::<Vec<_>>();
        let Some(index) = ui.choose("Select model (type to search)", &items, 0)? else {
            return Ok(());
        };
        set_model(session, media, rows[index].clone());
    }
    Ok(())
}
fn set_model(session: &mut Session, media: Media, model: catalog::Model) {
    let selection = Selection {
        provider: model.provider,
        model: model.model,
    };
    if session.config.defaults.get(&media) != Some(&selection) {
        session.config.defaults.insert(media, selection);
        session
            .config
            .default_settings
            .insert(media, defaults(media, model.provider));
        session.config.default_locales.remove(&media);
    }
}
fn keys(ui: &Ui, session: &mut Session, root: &Path) -> Result<()> {
    let providers = session
        .config
        .defaults
        .values()
        .map(|s| s.provider)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if providers.is_empty() {
        ui.message("Enable a feature first", false);
        return Ok(());
    }
    let items = providers
        .iter()
        .map(|p| {
            format!(
                "{} [{}]",
                p.key(),
                if session.pending_keys.contains_key(p) || credentials::resolve(*p, root).is_ok() {
                    "present; never displayed"
                } else {
                    "missing or invalid"
                }
            )
        })
        .collect::<Vec<_>>();
    let Some(i) = ui.choose("Project credentials — no network checks", &items, 0)? else {
        return Ok(());
    };
    let provider = providers[i];
    if credentials::resolve(provider, root).is_ok() {
        ui.message(
            "Existing credential preserved. Environment values take precedence.",
            true,
        );
        return Ok(());
    }
    if std::env::var_os(provider.key()).is_some() {
        ui.message(
            "Correct or unset the empty process environment value first; .env cannot override it",
            false,
        );
        return Ok(());
    }
    let value = dialoguer::Password::with_theme(&ui.theme)
        .with_prompt(format!(
            "{} (hidden; blank keeps it missing)",
            provider.key()
        ))
        .allow_empty_password(true)
        .interact_on(&ui.term)
        .map_err(|_| {
            Error::new(
                "setup_cancelled",
                "Credential entry cancelled; unsaved choices were discarded",
                130,
            )
        })?;
    if !value.is_empty() {
        if value.chars().any(char::is_control) {
            ui.message("Credential cannot contain control characters", false);
        } else {
            session.pending_keys.insert(provider, value);
        }
    }
    Ok(())
}
fn voice(ui: &Ui, session: &mut Session) -> Result<()> {
    let choices = options(&[
        "Set speech voice and reuse for podcasts",
        "Set a separate podcast voice",
    ]);
    let Some(i) = ui.choose(
        "Voice bindings — paste an existing provider voice ID",
        &choices,
        0,
    )?
    else {
        return Ok(());
    };
    let media = if i == 0 {
        Media::Speech
    } else {
        Media::Podcast
    };
    let Some(selection) = session.config.defaults.get(&media).cloned() else {
        ui.message("Enable this feature first", false);
        return Ok(());
    };
    let old = session
        .config
        .default_settings
        .get(&media)
        .and_then(|s| s.voice.as_deref())
        .unwrap_or("");
    let value = ui.input("Voice ID (blank clears); no voice is created", old)?;
    if !value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        ui.message(
            "Voice ID must contain only letters, numbers, underscores or hyphens",
            false,
        );
        return Ok(());
    }
    let binding = if value.is_empty() { None } else { Some(value) };
    session
        .config
        .default_settings
        .entry(media)
        .or_default()
        .voice = binding.clone();
    if i == 0
        && session
            .config
            .defaults
            .get(&Media::Podcast)
            .is_some_and(|p| p.provider == selection.provider && p.model == selection.model)
    {
        session
            .config
            .default_settings
            .entry(Media::Podcast)
            .or_default()
            .voice = binding;
    }
    Ok(())
}
fn advanced(ui: &Ui, session: &mut Session) -> Result<()> {
    let Some(section) = ui.choose(
        "Advanced settings",
        &options(&["Generation settings", "Run limits", "Identity policy"]),
        0,
    )?
    else {
        return Ok(());
    };
    if section == 2 {
        let Some(require) = ui.choose(
            "Require known origins for external identity references?",
            &options(&["Yes (recommended)", "No"]),
            if session.config.consistency.require_known_identity_model {
                0
            } else {
                1
            },
        )?
        else {
            return Ok(());
        };
        let Some(inherit) = ui.choose(
            "Inherit compatible source settings?",
            &options(&["Yes (recommended)", "No"]),
            if session.config.consistency.inherit_source_settings {
                0
            } else {
                1
            },
        )?
        else {
            return Ok(());
        };
        session.config.consistency.require_known_identity_model = require == 0;
        session.config.consistency.inherit_source_settings = inherit == 0;
    } else if section == 1 {
        let old = session.config.limits.clone();
        let mut value = serde_json::to_value(&old)?;
        let items = value
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        if let Some(i) = ui.choose("Choose a run limit", &items, 0)? {
            let key = &items[i];
            let s = ui.input(key, &value[key].to_string())?;
            if let Ok(n) = s.parse::<u64>() {
                value[key] = n.into();
                let limits: Limits = serde_json::from_value(value)?;
                match validation::limits(&limits) {
                    Ok(()) => session.config.limits = limits,
                    Err(e) => ui.message(&e.to_string(), false),
                }
            } else {
                ui.message("Enter a whole number", false);
            }
        }
    } else {
        let enabled = session.config.defaults.keys().copied().collect::<Vec<_>>();
        if enabled.is_empty() {
            return Ok(());
        }
        if let Some(i) = ui.choose(
            "Feature settings",
            &enabled.iter().map(|m| format!("{m:?}")).collect::<Vec<_>>(),
            0,
        )? {
            let media = enabled[i];
            let mut candidate = session.config.clone();
            let target = catalog::target(media);
            let provider = candidate.defaults[&media].provider;
            let settings = candidate.default_settings.entry(media).or_default();
            if target == Media::Image && provider == Provider::Openai {
                if let Some(i) = ui.choose(
                    "Image shape",
                    &options(&["1024x1024", "1024x1536", "1536x1024", "auto"]),
                    0,
                )? {
                    settings.size = Some(["1024x1024", "1024x1536", "1536x1024", "auto"][i].into());
                } else {
                    return Ok(());
                }
                let qualities = if candidate.defaults[&media]
                    .model
                    .starts_with("gpt-image-2.5-")
                {
                    options(&["high", "medium", "low", "auto", "xhigh", "max"])
                } else {
                    options(&["high", "medium", "low", "auto"])
                };
                if let Some(i) = ui.choose("Image quality", &qualities, 0)? {
                    settings.image_quality = Some(qualities[i].clone());
                } else {
                    return Ok(());
                }
            } else if matches!(target, Media::Music | Media::Sfx) {
                if let Ok(seconds) = ui
                    .input(
                        "Duration in seconds",
                        &settings.duration_seconds.unwrap_or(10).to_string(),
                    )?
                    .parse::<u32>()
                {
                    settings.duration_seconds = Some(seconds);
                } else {
                    ui.message("Enter a whole number", false);
                    return Ok(());
                }
                if target == Media::Music {
                    let Some(style) =
                        ui.choose("Music style", &options(&["Instrumental", "Vocal"]), 0)?
                    else {
                        return Ok(());
                    };
                    settings.instrumental = Some(style == 0);
                }
            } else if target == Media::Speech
                && !matches!(provider, Provider::Fish | Provider::Elevenlabs)
            {
                let locale = ui.input(
                    "Locale (blank omits)",
                    candidate
                        .default_locales
                        .get(&media)
                        .map(String::as_str)
                        .unwrap_or(""),
                )?;
                if locale.is_empty() {
                    candidate.default_locales.remove(&media);
                } else {
                    candidate.default_locales.insert(media, locale);
                }
            } else {
                ui.message(
                    "No additional settings for this feature; voice IDs are in the Voices section",
                    true,
                );
            }
            match validation::config(&candidate) {
                Ok(()) => session.config = candidate,
                Err(e) => ui.message(&e.to_string(), false),
            }
        }
    }
    Ok(())
}

pub fn run(
    root: &Path,
    initial: Config,
    agents: Vec<String>,
    color: crate::cli::Color,
) -> Result<serde_json::Value> {
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err(Error::input(
            "Setup requires a terminal; agents use init --preset recommended --json --non-interactive",
        ));
    }
    let ui = Ui::new(color);
    project::check_init_destination(root)?;
    eprintln!("Project folder: {}", absolute(root)?.display());
    let mut session = Session::new(initial, agents);
    let initial_hash = std::fs::read(crate::project::directory(root).join("config.yaml"))
        .ok()
        .map(|b| hash(&b));
    let initial_env_hash = std::fs::read(root.join(".env")).ok().map(|b| hash(&b));
    if initial_hash.is_none() {
        let Some(choice) = ui.choose(
            "Welcome to FrameChord — offline setup",
            &options(&[
                "Recommended setup — Google",
                "OpenAI setup — images, speech and planning",
                "Customize setup",
                "Empty project",
            ]),
            if session.config.defaults.is_empty() {
                3
            } else if session
                .config
                .defaults
                .get(&Media::Image)
                .is_some_and(|s| s.provider == Provider::Openai)
            {
                1
            } else {
                0
            },
        )?
        else {
            return Err(Error::new(
                "setup_cancelled",
                "Setup cancelled; no files changed",
                130,
            ));
        };
        if choice == 0 {
            session.config = recommended();
        }
        if choice == 1 {
            session.config = openai();
        }
        if choice == 3 {
            session.config = Config::default();
            session.config.consistency.require_known_identity_model = true;
        }
    }
    eprintln!(
        "Use arrows and Enter; type to search; Esc goes back. Credentials and provider access are not tested online."
    );
    loop {
        let report = session.report(root);
        for row in report["features"].as_array().unwrap() {
            ui.message(
                &format!(
                    "[{}] {} — {}",
                    if row["ready"] == true {
                        "ready"
                    } else {
                        "needs attention"
                    },
                    row["media"].as_str().unwrap_or(""),
                    row["selection"]["model"].as_str().unwrap_or("")
                ),
                row["ready"] == true,
            );
        }
        for issue in report["pending_steps"].as_array().unwrap() {
            ui.message(issue.as_str().unwrap_or(""), false);
        }
        let menu = options(&[
            "Features and models",
            "API keys",
            "Voices",
            "Agent skills",
            "Characters (optional)",
            "Advanced settings",
            "Review and finish setup",
            "Save unfinished setup and exit",
            "Discard changes and exit",
        ]);
        let action = ui.choose(
            "Setup checklist — return here after every section",
            &menu,
            if session.finishable(root) { 6 } else { 0 },
        )?;
        let result: Result<()> = match action {
            Some(0) => feature(&ui, &mut session, root),
            Some(1) => keys(&ui, &mut session, root),
            Some(2) => voice(&ui, &mut session),
            Some(3) => {
                if let Some(i) = ui.choose(
                    "Install project-local skills",
                    &options(&[
                        "Codex and Claude Code (also found by Cursor, Gemini CLI, OpenCode and Copilot)",
                        "Codex",
                        "Claude Code",
                        "Choose agents individually…",
                        "No skills (other process-based agent)",
                    ]),
                    0,
                )? {
                    session.agents = match i {
                        0 => default_agents(),
                        1 => vec!["codex".into()],
                        2 => vec!["claude".into()],
                        3 => choose_agents(&ui, session.agents.clone())?,
                        _ => vec![],
                    };
                }
                Ok(())
            }
            Some(4) => {
                if let Some(i) = ui.choose(
                    "Optional characters",
                    &options(&[
                        "Keep the optional persona scaffold",
                        "Inspect existing characters",
                        "Create an additional character scaffold on save",
                    ]),
                    0,
                )? {
                    if i == 1 {
                        let paths =
                            std::fs::read_dir(crate::project::directory(root).join("characters"))
                                .into_iter()
                                .flatten()
                                .flatten()
                                .map(|p| p.file_name().to_string_lossy().into_owned())
                                .collect::<Vec<_>>();
                        eprintln!(
                            "Existing character IDs: {:?}. Pending scaffolds: {:?}",
                            paths, session.personas
                        );
                    }
                    if i == 2 {
                        let name = ui.input("New stable character ID", "")?;
                        match crate::storage::id(&name){Ok(()) if !session.personas.contains(&name)&&!crate::project::directory(root).join("characters").join(&name).exists()&&name!="persona"=>session.personas.push(name),_=>ui.message("Choose a unique valid character ID; existing identities are preserved",false)}
                    }
                }
                eprintln!(
                    "Author the persona/visual/voice Markdown from the bundled templates after setup. Bootstrap and reference acceptance are separate explicit operations. No voices or reference candidates are created here."
                );
                Ok(())
            }
            Some(5) => advanced(&ui, &mut session),
            Some(6) | Some(7) => {
                if action == Some(6) && !session.finishable(root) {
                    ui.message("Finish is unavailable until every enabled feature is ready. Correct the checklist, disable an optional feature, or save unfinished setup.",false);
                    continue;
                }
                eprintln!(
                    "Project: {}\nSkills: {:?}\nConfiguration:\n{}",
                    root.display(),
                    session.agents,
                    serde_saphyr::to_string(&session.config)
                        .map_err(|_| Error::input("Cannot render configuration"))?
                );
                eprintln!(
                    "Credentials: names/status only; new values will be written to the ignored project .env. Existing characters are preserved."
                );
                if ui.choose(
                    "Save these reviewed choices?",
                    &options(&["Save", "Return to setup"]),
                    1,
                )? != Some(0)
                {
                    continue;
                }
                let now = std::fs::read(crate::project::directory(root).join("config.yaml"))
                    .ok()
                    .map(|b| hash(&b));
                if now != initial_hash
                    || std::fs::read(root.join(".env")).ok().map(|b| hash(&b)) != initial_env_hash
                {
                    return Err(Error::input(
                        "Configuration changed during setup; restart to review the current file",
                    ));
                }
                match session.commit(root, Some((&initial_hash, &initial_env_hash))) {
                    Ok(value) => return Ok(value),
                    Err(error) => {
                        ui.message(&error.to_string(), false);
                        continue;
                    }
                }
            }
            _ => {
                if ui.choose(
                    "Discard all unsaved changes?",
                    &options(&["Return to setup", "Discard and exit"]),
                    0,
                )? == Some(1)
                {
                    return Err(Error::new(
                        "setup_cancelled",
                        "Setup discarded; no unsaved choices were saved",
                        130,
                    ));
                }
                Ok(())
            }
        };
        if let Err(error) = result {
            if error.exit == 130 {
                return Err(error);
            }
            ui.message(&error.to_string(), false);
        }
    }
}
pub fn wizard(root: &Path, color: crate::cli::Color) -> Result<serde_json::Value> {
    let selected = init_destination(root, color)?;
    let root = selected.as_path();
    run(
        root,
        if crate::project::directory(root).join("config.yaml").exists() {
            project::config(root)?
        } else {
            recommended()
        },
        default_agents(),
        color,
    )
}
fn default_agents() -> Vec<String> {
    project::DEFAULT_AGENTS
        .iter()
        .map(|agent| (*agent).to_owned())
        .collect()
}
/// Flips one agent in or out of the selection; the `none` placeholder never survives a toggle.
fn toggle_agent(mut selected: Vec<String>, name: &str) -> Vec<String> {
    selected.retain(|chosen| chosen != "none");
    if let Some(position) = selected.iter().position(|chosen| chosen == name) {
        selected.remove(position);
    } else {
        selected.push(name.to_owned());
    }
    selected
}
/// Toggle individual agent targets with the single-select prompt; "Done" returns.
/// Toggle rows in catalog order plus Done, and the row to highlight: the first agent
/// currently selected, or Done when none is, so arrow keys start on the current choice.
fn agent_menu(selected: &[String]) -> (Vec<String>, usize) {
    let chosen = |target: &project::AgentTarget| selected.iter().any(|name| name == target.name);
    let mut items: Vec<String> = project::AGENT_TARGETS
        .iter()
        .map(|target| {
            let mark = if chosen(target) { "x" } else { " " };
            format!("[{mark}] {}", target.label)
        })
        .collect();
    let default = project::AGENT_TARGETS
        .iter()
        .position(chosen)
        .unwrap_or(items.len());
    items.push("Done".into());
    (items, default)
}
fn choose_agents(ui: &Ui, mut selected: Vec<String>) -> Result<Vec<String>> {
    selected.retain(|chosen| chosen != "none");
    loop {
        let (items, default) = agent_menu(&selected);
        match ui.choose("Toggle agents; choose Done to return", &items, default)? {
            Some(i) if i < project::AGENT_TARGETS.len() => {
                selected = toggle_agent(selected, project::AGENT_TARGETS[i].name);
            }
            _ => return Ok(selected),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toggling_an_agent_drops_the_none_placeholder_and_flips_membership() {
        let selected = toggle_agent(vec!["none".into()], "codex");
        assert_eq!(selected, vec!["codex".to_string()]);
        let selected = toggle_agent(vec!["codex".into(), "claude".into()], "codex");
        assert_eq!(selected, vec!["claude".to_string()]);
        let selected = toggle_agent(selected, "copilot");
        assert_eq!(selected, vec!["claude".to_string(), "copilot".to_string()]);
    }
    #[test]
    fn agent_menu_highlights_the_first_selected_agent_or_done() {
        let (items, default) = agent_menu(&["copilot".into()]);
        assert_eq!(items[default], "[x] GitHub Copilot");
        assert_eq!(items.last().unwrap(), "Done");
        let (items, default) = agent_menu(&["claude".into(), "codex".into()]);
        assert_eq!(
            items[default], "[x] Codex",
            "menu order, not selection order"
        );
        let (items, default) = agent_menu(&["claude".into()]);
        assert_eq!(items[default], "[x] Claude Code");
        assert_eq!(items[0], "[ ] Codex");
        let (items, default) = agent_menu(&[]);
        assert_eq!(items[default], "Done");
        assert_eq!(default, project::AGENT_TARGETS.len());
    }
    #[test]
    fn reviewed_save_rejects_stale_files_without_losing_credentials() {
        let root = tempfile::tempdir().unwrap();
        project::init(root.path(), &[]).unwrap();
        let config_file = root.path().join("framechord/config.yaml");
        let before = std::fs::read(&config_file).unwrap();
        let config_hash = Some(hash(&before));
        let env_hash = Some(hash(&std::fs::read(root.path().join(".env")).unwrap()));
        std::fs::write(root.path().join(".env"), "OTHER=newer-user-content\n").unwrap();
        let session = Session::new(recommended(), vec![]);
        assert!(
            session
                .commit(root.path(), Some((&config_hash, &env_hash)))
                .is_err()
        );
        assert_eq!(std::fs::read(config_file).unwrap(), before);
        assert_eq!(
            std::fs::read_to_string(root.path().join(".env")).unwrap(),
            "OTHER=newer-user-content\n"
        );
    }
    #[test]
    fn every_selectable_model_starts_with_compatible_generation_settings() {
        for model in catalog::models()
            .into_iter()
            .filter(|m| m.current && m.supported)
        {
            let mut config = Config::default();
            config.defaults.insert(
                model.media,
                Selection {
                    provider: model.provider,
                    model: model.model.clone(),
                },
            );
            let settings = defaults(model.media, model.provider);
            if model.media != Media::Text {
                config.default_settings.insert(model.media, settings);
            }
            validation::config(&config).unwrap_or_else(|error| panic!("{}: {error}", model.model));
        }
    }

    #[test]
    fn renamed_host_settings_keep_existing_limits_and_file_bytes() {
        let home = tempfile::tempdir().unwrap();
        let legacy = home.path().join(".config/mediaagent/host.yaml");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        let bytes =
            b"# User-selected host ceiling\nschema_version: 1\nconcurrency: 1\nper_provider: 1\n";
        std::fs::write(&legacy, bytes).unwrap();
        let selected = host_path_in(home.path()).unwrap();
        let limits: HostLimits = yaml(&selected).unwrap();
        validate_host(&limits).unwrap();
        assert_eq!(limits.concurrency, 1);
        assert_eq!(limits.per_provider, 1);
        assert_eq!(std::fs::read(&selected).unwrap(), bytes);
        assert_eq!(selected, legacy);
        assert!(!home.path().join(".config/framechord").exists());
    }

    #[test]
    fn conflicting_host_settings_fail_without_choosing_a_higher_ceiling() {
        let home = tempfile::tempdir().unwrap();
        for name in ["mediaagent", "framechord"] {
            let path = home.path().join(format!(".config/{name}/host.yaml"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "conflicting fixture").unwrap();
        }
        assert!(host_path_in(home.path()).is_err());
        for name in ["mediaagent", "framechord"] {
            assert_eq!(
                std::fs::read_to_string(home.path().join(format!(".config/{name}/host.yaml")))
                    .unwrap(),
                "conflicting fixture"
            );
        }
    }
}
