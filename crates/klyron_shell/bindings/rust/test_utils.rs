//! Test utilities for klyron_shell
use crate::types::ShellConfig;

pub fn test_config() -> ShellConfig {
    ShellConfig { version: "0.0.0".into() }
}
