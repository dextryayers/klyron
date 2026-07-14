//! Test utilities for klyron_plugin
use crate::types::PluginConfig;

pub fn test_config() -> PluginConfig {
    PluginConfig { version: "0.0.0".into() }
}
