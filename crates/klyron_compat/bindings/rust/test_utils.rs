//! Test utilities for klyron_compat
use crate::types::CompatConfig;

pub fn test_config() -> CompatConfig {
    CompatConfig { version: "0.0.0".into() }
}
