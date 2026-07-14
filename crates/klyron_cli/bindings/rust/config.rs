//! Configuration for klyron_cli

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Klyron::CliSettings {
    pub max_retries: u32,
    pub timeout_ms: u64,
    pub log_level: String,
}

impl Default for Klyron::CliSettings {
    fn default() -> Self {
        Self { max_retries: 3, timeout_ms: 5000, log_level: "info".into() }
    }
}

pub fn load_config(path: Option<&std::path::Path>) -> anyhow::Result<Klyron::CliSettings> {
    if let Some(p) = path {
        let content = std::fs::read_to_string(p)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(Klyron::CliSettings::default())
    }
}
