use serde::{Deserialize, Serialize};

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
