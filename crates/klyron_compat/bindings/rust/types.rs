//! Types for klyron_compat
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatConfig {
    pub version: String,
}

impl Default for CompatConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
