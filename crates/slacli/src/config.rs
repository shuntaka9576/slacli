use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelEntry {
    pub id: String,
    pub description: String,
}

// --- config.toml ---

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<HashMap<String, ProfileConfig>>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProfileConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<HashMap<String, ChannelEntry>>,
}

fn config_dir() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").expect("HOME not set");
        format!("{home}/.config")
    });
    PathBuf::from(base).join("slacli")
}

impl Config {
    pub fn config_path() -> PathBuf {
        config_dir().join("config.toml")
    }

    pub fn load() -> Result<Self, AppError> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| AppError::Config(format!("Failed to read {}: {e}", path.display())))?;
        toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse {}: {e}", path.display())))
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Config(format!("Failed to create {}: {e}", parent.display()))
            })?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {e}")))?;
        write_file_0600(&path, &content)
    }

    pub fn resolve_profile_name(&self, cli_profile: Option<&str>) -> Result<String, AppError> {
        if let Some(name) = cli_profile {
            return Ok(name.to_string());
        }
        self.default_profile.clone().ok_or_else(|| {
            AppError::Config(
                "No profile specified. Use --profile or set default_profile in config.toml."
                    .to_string(),
            )
        })
    }

    pub fn get_profile(&self, name: &str) -> Result<&ProfileConfig, AppError> {
        self.profiles
            .as_ref()
            .and_then(|p| p.get(name))
            .ok_or_else(|| AppError::Config(format!("Profile '{name}' not found in config.toml.")))
    }

    pub fn resolve_profile(
        &self,
        credentials: &Credentials,
        cli_profile: Option<&str>,
    ) -> Result<ResolvedProfile, AppError> {
        let name = self.resolve_profile_name(cli_profile)?;
        let profile_config = self.get_profile(&name)?;
        let profile_creds = credentials.get_profile(&name)?;

        Ok(ResolvedProfile {
            bot_token: profile_creds.bot_token.clone(),
            user_token: profile_creds.user_token.clone(),
            channels: profile_config.channels.clone(),
        })
    }

    pub fn resolve_editor(&self) -> String {
        self.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "vim".to_string())
    }
}

// --- credentials.toml ---

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Credentials {
    #[serde(default)]
    pub profiles: HashMap<String, ProfileCredentials>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProfileCredentials {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_token: Option<String>,
}

impl Credentials {
    pub fn credentials_path() -> PathBuf {
        config_dir().join("credentials.toml")
    }

    pub fn load() -> Result<Self, AppError> {
        let path = Self::credentials_path();
        if !path.exists() {
            return Ok(Credentials::default());
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| AppError::Config(format!("Failed to read {}: {e}", path.display())))?;
        toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse {}: {e}", path.display())))
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = Self::credentials_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Config(format!("Failed to create {}: {e}", parent.display()))
            })?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize credentials: {e}")))?;
        write_file_0600(&path, &content)
    }

    pub fn get_profile(&self, name: &str) -> Result<&ProfileCredentials, AppError> {
        self.profiles.get(name).ok_or_else(|| {
            AppError::Config(format!(
                "Profile '{name}' not found in credentials.toml. Run 'slacli init'."
            ))
        })
    }
}

// --- ResolvedProfile ---

pub struct ResolvedProfile {
    pub bot_token: Option<String>,
    pub user_token: Option<String>,
    pub channels: Option<HashMap<String, ChannelEntry>>,
}

impl ResolvedProfile {
    pub fn bot_token(&self) -> Result<&str, AppError> {
        self.bot_token.as_deref().ok_or_else(|| {
            AppError::Config("Bot token not configured. Run 'slacli init'.".to_string())
        })
    }

    pub fn user_token(&self) -> Result<&str, AppError> {
        self.user_token.as_deref().ok_or_else(|| {
            AppError::Config("User token not configured. Run 'slacli init'.".to_string())
        })
    }

    pub fn resolve_channel(&self, input: &str) -> String {
        self.channels
            .as_ref()
            .and_then(|chs| chs.get(input))
            .map(|entry| entry.id.clone())
            .unwrap_or_else(|| input.to_string())
    }
}

// --- helpers ---

fn write_file_0600(path: &PathBuf, content: &str) -> Result<(), AppError> {
    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(content.as_bytes())
        })
        .map_err(|e| AppError::Config(format!("Failed to write {}: {e}", path.display())))
}
