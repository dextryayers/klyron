//! Types for klyron_workspace
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub version: String,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self { version: "0.1.0".into() }
    }
}
