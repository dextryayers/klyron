//! Builder for klyron_plugin
use crate::types::PluginConfig;

pub struct PluginBuilder {
    config: PluginConfig,
}

impl PluginBuilder {
    pub fn new() -> Self {
        Self { config: PluginConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> PluginConfig {
        self.config
    }
}
