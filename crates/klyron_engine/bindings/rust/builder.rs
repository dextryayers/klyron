//! Builder pattern for klyron_engine

use crate::types::Klyron::EngineConfig;

#[derive(Debug, Default)]
pub struct Klyron::EngineBuilder {
    config: Option<Klyron::EngineConfig>,
    verbose: bool,
}

impl Klyron::EngineBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Klyron::EngineConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(self) -> anyhow::Result<Klyron::EngineInstance> {
        let config = self.config.unwrap_or_default();
        Ok(Klyron::EngineInstance { config, verbose: self.verbose })
    }
}

#[derive(Debug)]
pub struct Klyron::EngineInstance {
    pub config: Klyron::EngineConfig,
    pub verbose: bool,
}
