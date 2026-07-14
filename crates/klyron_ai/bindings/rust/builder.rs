//! Builder pattern for klyron_ai

use crate::types::Klyron::AiConfig;

#[derive(Debug, Default)]
pub struct Klyron::AiBuilder {
    config: Option<Klyron::AiConfig>,
    verbose: bool,
}

impl Klyron::AiBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Klyron::AiConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(self) -> anyhow::Result<Klyron::AiInstance> {
        let config = self.config.unwrap_or_default();
        Ok(Klyron::AiInstance { config, verbose: self.verbose })
    }
}

#[derive(Debug)]
pub struct Klyron::AiInstance {
    pub config: Klyron::AiConfig,
    pub verbose: bool,
}
