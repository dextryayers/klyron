//! Types for klyron_shell
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub version: String,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
