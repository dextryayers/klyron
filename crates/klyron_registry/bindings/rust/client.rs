pub struct RegistryClient {
    config: crate::config::RegistryConfig,
}

impl RegistryClient {
    pub fn new() -> Self { Self { config: crate::config::RegistryConfig::default() } }
    pub fn config(&self) -> &crate::config::RegistryConfig { &self.config }
    pub fn version(&self) -> &'static str { "1.0.0" }
}
