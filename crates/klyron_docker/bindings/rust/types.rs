//! Types for klyron_docker
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub version: String,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
