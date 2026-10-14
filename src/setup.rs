//! Explicit local configuration and terminal-only authoring assistance.
use crate::{
    contracts::*,
    error::{Error, Result},
    project,
    storage::*,
    validation,
};
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
        .ok_or_else(|| Error::input("Cannot locate user home for mediaagent host settings"))?;
    Ok(PathBuf::from(home).join(".config/mediaagent/host.yaml"))
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
fn answer(prompt: &str) -> Result<String> {
    eprint!("{prompt}");
    io::stderr().flush()?;
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Err(Error::input("Input ended before configuration was saved"));
    }
    Ok(line.trim().to_owned())
}
fn field(label: &str, current: Option<&str>) -> Result<Option<String>> {
    let value = answer(&format!(
        "{label} [{}; '-' clears]: ",
        current.unwrap_or("unset")
    ))?;
    Ok(match value.as_str() {
        "" => current.map(str::to_owned),
        "-" => None,
        _ => Some(value),
    })
}
fn number<T: std::str::FromStr + std::fmt::Display>(label: &str, current: T) -> Result<T> {
    let value = answer(&format!("{label} [{current}]: "))?;
    if value.is_empty() {
        return Ok(current);
    }
    value.parse().map_err(|_| {
        Error::input(format!(
            "{label} requires a whole number; configuration was not saved"
        ))
    })
}
fn toggle(label: &str, current: bool) -> Result<bool> {
    match answer(&format!(
        "{label} [{}]: ",
        if current { "Y/n" } else { "y/N" }
    ))?
    .to_ascii_lowercase()
    .as_str()
    {
        "" => Ok(current),
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Err(Error::input(
            "Answer yes or no; configuration was not saved",
        )),
    }
}
/// Collect and validate a complete configuration before the caller writes any files.
pub fn configure(mut config: Config) -> Result<Config> {
    if !io::stdin().is_terminal() {
        return Err(Error::input(
            "Configuration wizard requires a terminal; use init --defaults-file or config setup --file for noninteractive callers",
        ));
    }
    eprintln!(
        "Offline setup. Enter keeps the displayed value; '-' clears an optional value. No credentials or provider calls are needed. Known identity originals always pin their provider/model."
    );
    for media in [
        Media::Image,
        Media::Infographic,
        Media::Speech,
        Media::Podcast,
        Media::Music,
        Media::Sfx,
        Media::Text,
    ] {
        if media == Media::Infographic
            && !config.defaults.contains_key(&media)
            && config.defaults.contains_key(&Media::Image)
            && toggle("Use image model and settings for infographics?", true)?
        {
            config
                .defaults
                .insert(media, config.defaults[&Media::Image].clone());
            if let Some(settings) = config.default_settings.get(&Media::Image).cloned() {
                config.default_settings.insert(media, settings);
            }
            continue;
        }
        let current = config.defaults.get(&media).cloned();
        let label = format!("{media:?}");
        let provider = field(
            &format!("{label} provider (openai/google/fish/openrouter/elevenlabs)"),
            current.as_ref().map(|s| match s.provider {
                Provider::Openai => "openai",
                Provider::Google => "google",
                Provider::Fish => "fish",
                Provider::Openrouter => "openrouter",
                Provider::Elevenlabs => "elevenlabs",
            }),
        )?;
        let Some(provider) = provider else {
            config.defaults.remove(&media);
            config.default_settings.remove(&media);
            config.default_locales.remove(&media);
            continue;
        };
        let provider: Provider = serde_json::from_value(serde_json::json!(provider))
            .map_err(|_| Error::input("Unknown provider; configuration was not saved"))?;
        let target = match media {
            Media::Infographic => Media::Image,
            Media::Podcast => Media::Speech,
            other => other,
        };
        let choices = crate::providers::registry()
            .into_iter()
            .filter(|c| c.provider == provider && c.media == target)
            .map(|c| c.model)
            .collect::<Vec<_>>();
        if target != Media::Text {
            eprintln!("Supported model IDs: {}", choices.join(", "));
        }
        let model = field(
            "Explicit model ID",
            current
                .as_ref()
                .filter(|s| s.provider == provider)
                .map(|s| s.model.as_str()),
        )?
        .ok_or_else(|| Error::input("A selected provider needs an explicit model ID"))?;
        if target != Media::Text && !choices.contains(&model.as_str()) {
            return Err(Error::input(
                "Unsupported provider/model; configuration was not saved",
            ));
        }
        let selection = Selection { provider, model };
        if current.as_ref() != Some(&selection) {
            config.default_settings.remove(&media);
            config.default_locales.remove(&media);
        }
        let mut settings = config
            .default_settings
            .get(&media)
            .cloned()
            .unwrap_or_default();
        match target {
            Media::Image => {
                settings.format = Some("png".into());
                if provider == Provider::Openai {
                    settings.size = field(
                        "Image size: 1024x1024/1024x1536/1536x1024/auto",
                        settings.size.as_deref().or(Some("1024x1024")),
                    )?;
                    settings.image_quality = field(
                        "Image quality: low/medium/high/auto",
                        settings.image_quality.as_deref().or(Some("high")),
                    )?;
                    let fidelity = crate::providers::fidelity_values(&selection.model);
                    if !fidelity.is_empty() {
                        settings.input_fidelity = field(
                            "Reference edit fidelity (identity requires high)",
                            settings
                                .input_fidelity
                                .as_deref()
                                .or(fidelity.first().copied()),
                        )?;
                    }
                }
            }
            Media::Speech => {
                settings.format = Some("wav".into());
                settings.voice = field(
                    "Saved provider voice ID or built-in voice (may configure later)",
                    settings.voice.as_deref(),
                )?;
                if !matches!(provider, Provider::Fish | Provider::Elevenlabs)
                    && !matches!(selection.model.as_str(), "tts-1" | "tts-1-hd")
                {
                    let locale = field(
                        "Locale direction, for example en-US",
                        config.default_locales.get(&media).map(String::as_str),
                    )?;
                    if let Some(locale) = locale {
                        config.default_locales.insert(media, locale);
                    } else {
                        config.default_locales.remove(&media);
                    }
                }
            }
            Media::Music | Media::Sfx => {
                settings.format = Some("mp3".into());
                let duration = if target == Media::Sfx {
                    3
                } else if selection.model == "lyria-3-clip-preview" {
                    30
                } else if selection.model == "lyria-3.5" {
                    60
                } else {
                    10
                };
                settings.duration_seconds = Some(number(
                    "Duration in seconds",
                    settings.duration_seconds.unwrap_or(duration),
                )?);
                if target == Media::Music {
                    settings.instrumental = Some(toggle(
                        "Instrumental music?",
                        settings.instrumental.unwrap_or(true),
                    )?);
                }
            }
            _ => {}
        }
        config.defaults.insert(media, selection);
        if target != Media::Text {
            config.default_settings.insert(media, settings);
        }
    }
    config.consistency.require_known_identity_model = toggle(
        "Require recorded or declared model for external identity references?",
        config.consistency.require_known_identity_model,
    )?;
    config.consistency.inherit_source_settings = toggle(
        "Inherit the original asset's compatible settings?",
        config.consistency.inherit_source_settings,
    )?;
    config.limits.requests = number("Maximum requests per run", config.limits.requests)?;
    config.limits.images = number("Maximum images per run", config.limits.images)?;
    config.limits.audio_seconds =
        number("Maximum audio seconds per run", config.limits.audio_seconds)?;
    config.limits.speech_chars = number(
        "Maximum speech characters per run",
        config.limits.speech_chars,
    )?;
    config.limits.revisions = number("Maximum revisions", config.limits.revisions)?;
    config.limits.attempts = number("Maximum attempts (1–3)", config.limits.attempts)?;
    config.limits.concurrency = number("Parallel jobs (1–32)", config.limits.concurrency)?;
    config.limits.per_provider = number(
        "Parallel jobs per provider (1–16)",
        config.limits.per_provider,
    )?;
    validation::config(&config)?;
    eprintln!(
        "{}",
        serde_saphyr::to_string(&config)
            .map_err(|_| Error::input("Cannot render configuration"))?
    );
    if !toggle("Save this configuration?", false)? {
        return Err(Error::input("Configuration not saved"));
    }
    Ok(config)
}
pub fn wizard(root: &Path) -> Result<Config> {
    let config = configure(project::config(root)?)?;
    save(&root.join("mediaagent/config.yaml"), &config)?;
    Ok(config)
}
