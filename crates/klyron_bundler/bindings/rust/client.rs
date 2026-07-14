pub struct BundlerClient {
    config: crate::config::BundlerConfig,
}

impl BundlerClient {
    pub fn new() -> Self { Self { config: crate::config::BundlerConfig::default() } }
    pub fn config(&self) -> &crate::config::BundlerConfig { &self.config }
    pub fn version(&self) -> &'static str { "1.0.0" }
}
