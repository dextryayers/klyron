//! Builder pattern for klyron_runtime

use crate::types::Klyron::RuntimeConfig;

#[derive(Debug, Default)]
pub struct Klyron::RuntimeBuilder {
    config: Option<Klyron::RuntimeConfig>,
    verbose: bool,
}

impl Klyron::RuntimeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Klyron::RuntimeConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(self) -> anyhow::Result<Klyron::RuntimeInstance> {
        let config = self.config.unwrap_or_default();
        Ok(Klyron::RuntimeInstance { config, verbose: self.verbose })
    }
}

#[derive(Debug)]
pub struct Klyron::RuntimeInstance {
    pub config: Klyron::RuntimeConfig,
    pub verbose: bool,
}
