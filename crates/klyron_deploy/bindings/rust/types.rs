//! Types for klyron_deploy
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub version: String,
}

impl Default for DeployConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
