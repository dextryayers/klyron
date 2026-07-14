//! Builder for klyron_config
use crate::types::ConfigConfig;

pub struct ConfigBuilder {
    config: ConfigConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self { config: ConfigConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> ConfigConfig {
        self.config
    }
}
