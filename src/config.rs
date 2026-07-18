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
        Self {
            api_key: None,
            refresh_interval_minutes: 30,
            auto_start: false,
        }
    }
}

/// Type alias for the dialog callback.
pub type DialogFn = fn(&str, &str, &str) -> Option<String>;

impl Config {
    /// Default config directory: %APPDATA%/deepseek-tray
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("deepseek-tray")
    }

    /// Default config file path.
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Load config from a specific path (for testing).
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("无法读取配置文件: {}", path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("配置文件格式错误: {}", path.display()))
        } else {
            Ok(Config::default())
        }
    }

    /// Load config from the default path.
    pub fn load() -> Result<Self> {
        Self::load_from_path(&Self::config_path())
    }

    /// Save config to a specific path (for testing).
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建配置目录: {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self).context("序列化配置失败")?;
        fs::write(path, content)
            .with_context(|| format!("无法写入配置文件: {}", path.display()))?;
        Ok(())
    }

    /// Save config to the default path.
    pub fn save(&self) -> Result<()> {
        self.save_to_path(&Self::config_path())
    }

    /// Resolve the API key with priority: env var → config file → optional dialog.
    pub fn resolve_api_key(
        &mut self,
        config_path: &Path,
        dialog_fn: Option<DialogFn>,
    ) -> Result<String> {
        // 1. Environment variable (highest priority)
        if let Ok(key) = std::env::var("DEEPSEEK_API_KEY") {
            let key = key.trim();
            if !key.is_empty() {
                log::info!("Using API key from DEEPSEEK_API_KEY env var");
                return Ok(key.to_string());
            }
        }

        // 2. Config file
        if let Some(ref key) = self.api_key {
            let key = key.trim();
            if !key.is_empty() {
                log::info!("Using API key from config file");
                return Ok(key.to_string());
            }
        }

        // 3. Dialog fallback
        if let Some(dialog) = dialog_fn {
            if let Some(key) = dialog(
                "DeepSeek Tray — 配置 API Key",
                "请输入 DeepSeek API Key (sk-...):",
                "",
            ) {
                let key = key.trim().to_string();
                if !key.is_empty() {
                    self.api_key = Some(key.clone());
                    self.save_to_path(config_path)?;
                    log::info!("API key saved from user input");
                    return Ok(key);
                }
            }
        }

        Err(anyhow!(
            "未配置 DeepSeek API Key。请设置环境变量 DEEPSEEK_API_KEY，\n\
             或编辑配置文件 {}，或在弹出的对话框中输入。",
            config_path.display()
        ))
    }
}
