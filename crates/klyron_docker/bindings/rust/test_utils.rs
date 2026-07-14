//! Test utilities for klyron_docker
use crate::types::DockerConfig;

pub fn test_config() -> DockerConfig {
    DockerConfig { version: "0.0.0".into() }
}
