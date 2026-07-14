//! Builder for klyron_adapter
use crate::types::AdapterConfig;

pub struct AdapterBuilder {
    config: AdapterConfig,
}

impl AdapterBuilder {
    pub fn new() -> Self {
        Self { config: AdapterConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> AdapterConfig {
        self.config
    }
}
