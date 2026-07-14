//! Builder pattern for klyron_updater

use crate::types::Klyron::UpdaterConfig;

#[derive(Debug, Default)]
pub struct Klyron::UpdaterBuilder {
    config: Option<Klyron::UpdaterConfig>,
    verbose: bool,
}

impl Klyron::UpdaterBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Klyron::UpdaterConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn build(self) -> anyhow::Result<Klyron::UpdaterInstance> {
        let config = self.config.unwrap_or_default();
        Ok(Klyron::UpdaterInstance { config, verbose: self.verbose })
    }
}

#[derive(Debug)]
pub struct Klyron::UpdaterInstance {
    pub config: Klyron::UpdaterConfig,
    pub verbose: bool,
}
