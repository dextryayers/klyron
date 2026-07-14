//! Builder for klyron_compat
use crate::types::CompatConfig;

pub struct CompatBuilder {
    config: CompatConfig,
}

impl CompatBuilder {
    pub fn new() -> Self {
        Self { config: CompatConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> CompatConfig {
        self.config
    }
}
