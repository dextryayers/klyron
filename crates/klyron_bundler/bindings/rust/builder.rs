use crate::config::BundlerConfig;

pub struct BundlerBuilder {
    config: BundlerConfig,
}

impl BundlerBuilder {
    pub fn new() -> Self { Self { config: BundlerConfig::default() } }
    pub fn build(self) -> BundlerConfig { self.config }
}

impl Default for BundlerBuilder {
    fn default() -> Self { Self::new() }
}
