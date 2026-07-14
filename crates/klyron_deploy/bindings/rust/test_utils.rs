//! Test utilities for klyron_deploy
use crate::types::DeployConfig;

pub fn test_config() -> DeployConfig {
    DeployConfig { version: "0.0.0".into() }
}
