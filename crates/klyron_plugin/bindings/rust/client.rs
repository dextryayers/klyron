//! Client for klyron_plugin
use crate::types::PluginConfig;

pub struct PluginClient {
    config: PluginConfig,
}

impl PluginClient {
    pub fn new(config: PluginConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
