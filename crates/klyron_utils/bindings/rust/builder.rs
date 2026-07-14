//! Builder pattern for klyron_utils

use crate::types::Klyron::UtilsConfig;

#[derive(Debug, Default)]
pub struct Klyron::UtilsBuilder {
    config: Option<Klyron::UtilsConfig>,
    verbose: bool,
}

impl Klyron::UtilsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Klyron::UtilsConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(self) -> anyhow::Result<Klyron::UtilsInstance> {
        let config = self.config.unwrap_or_default();
        Ok(Klyron::UtilsInstance { config, verbose: self.verbose })
    }
}

#[derive(Debug)]
pub struct Klyron::UtilsInstance {
    pub config: Klyron::UtilsConfig,
    pub verbose: bool,
}
