//! Types for klyron_adapter
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub version: String,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
