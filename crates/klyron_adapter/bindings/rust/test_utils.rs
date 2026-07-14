//! Test utilities for klyron_adapter
use crate::types::AdapterConfig;

pub fn test_config() -> AdapterConfig {
    AdapterConfig { version: "0.0.0".into() }
}
