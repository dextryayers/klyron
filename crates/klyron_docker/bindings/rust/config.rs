//! Config for klyron_docker
use crate::types::DockerConfig;

pub fn load_config() -> DockerConfig {
    DockerConfig::default()
}
