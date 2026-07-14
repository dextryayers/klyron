//! Builder for klyron_workspace
use crate::types::WorkspaceConfig;

pub struct WorkspaceBuilder {
    config: WorkspaceConfig,
}

impl WorkspaceBuilder {
    pub fn new() -> Self {
        Self { config: WorkspaceConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> WorkspaceConfig {
        self.config
    }
}
