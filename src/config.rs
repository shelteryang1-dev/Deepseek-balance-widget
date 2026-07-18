use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default = "default_interval")]
    pub refresh_interval_minutes: u32,
    #[serde(default)]
    pub auto_start: bool,
}

fn default_interval() -> u32 {
    30
}

impl Default for Config {
    fn default() -> Self {
        Self { api_key: None, refresh_interval_minutes: 30, auto_start: false }
    }
}

/// Dialog callback signature: (title, prompt, default) -> user_input.
pub type DialogFn = fn(&str, &str, &str) -> Option<String>;

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("deepseek-tray")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("failed to read config: {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("invalid config: {}", path.display()))
        } else {
            Ok(Config::default())
        }
    }

    pub fn load() -> Result<Self> {
        Self::load_from_path(&Self::config_path())
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create dir: {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self).context("serialization failed")?;
        fs::write(path, content)
            .with_context(|| format!("failed to write config: {}", path.display()))?;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        self.save_to_path(&Self::config_path())
    }

    /// Resolve API key: env var DEEPSEEK_API_KEY → config file → optional dialog.
    pub fn resolve_api_key(
        &mut self,
        config_path: &Path,
        dialog_fn: Option<DialogFn>,
    ) -> Result<String> {
        if let Ok(key) = std::env::var("DEEPSEEK_API_KEY") {
            let key = key.trim();
            if !key.is_empty() {
                log::info!("using API key from env var");
                return Ok(key.to_string());
            }
        }

        if let Some(ref key) = self.api_key {
            let key = key.trim();
            if !key.is_empty() {
                log::info!("using API key from config file");
                return Ok(key.to_string());
            }
        }

        if let Some(dialog) = dialog_fn {
            if let Some(key) = dialog("DeepSeek Tray", "Enter API key (sk-...):", "") {
                let key = key.trim().to_string();
                if !key.is_empty() {
                    self.api_key = Some(key.clone());
                    self.save_to_path(config_path)?;
                    log::info!("API key saved from dialog");
                    return Ok(key);
                }
            }
        }

        Err(anyhow!(
            "No API key configured.\n\
             Set DEEPSEEK_API_KEY env var, or edit {}.",
            config_path.display()
        ))
    }
}
