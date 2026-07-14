//! Types for klyron_config
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigConfig {
    pub version: String,
}

impl Default for ConfigConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
