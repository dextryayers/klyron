//! Builder for klyron_docker
use crate::types::DockerConfig;

pub struct DockerBuilder {
    config: DockerConfig,
}

impl DockerBuilder {
    pub fn new() -> Self {
        Self { config: DockerConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> DockerConfig {
        self.config
    }
}
