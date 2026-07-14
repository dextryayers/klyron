use crate::config::LinterConfig;

pub struct LinterBuilder {
    config: LinterConfig,
}

impl LinterBuilder {
    pub fn new() -> Self { Self { config: LinterConfig::default() } }
    pub fn build(self) -> LinterConfig { self.config }
}

impl Default for LinterBuilder {
    fn default() -> Self { Self::new() }
}
