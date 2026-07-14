//! Types for klyron_plugin
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub version: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
