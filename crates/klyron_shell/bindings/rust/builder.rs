//! Builder for klyron_shell
use crate::types::ShellConfig;

pub struct ShellBuilder {
    config: ShellConfig,
}

impl ShellBuilder {
    pub fn new() -> Self {
        Self { config: ShellConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> ShellConfig {
        self.config
    }
}
