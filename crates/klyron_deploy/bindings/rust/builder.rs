//! Builder for klyron_deploy
use crate::types::DeployConfig;

pub struct DeployBuilder {
    config: DeployConfig,
}

impl DeployBuilder {
    pub fn new() -> Self {
        Self { config: DeployConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> DeployConfig {
        self.config
    }
}
