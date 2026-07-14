//! Test utilities for klyron_workspace
use crate::types::WorkspaceConfig;

pub fn test_config() -> WorkspaceConfig {
    WorkspaceConfig { version: "0.0.0".into() }
}
