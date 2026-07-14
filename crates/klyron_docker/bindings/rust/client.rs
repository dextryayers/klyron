//! Client for klyron_docker
use crate::types::DockerConfig;

pub struct DockerClient {
    config: DockerConfig,
}

impl DockerClient {
    pub fn new(config: DockerConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
