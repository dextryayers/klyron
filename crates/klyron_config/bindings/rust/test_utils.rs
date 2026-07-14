//! Test utilities for klyron_config
use crate::types::ConfigConfig;

pub fn test_config() -> ConfigConfig {
    ConfigConfig { version: "0.0.0".into() }
}
