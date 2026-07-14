use crate::config::RegistryConfig;

pub struct RegistryBuilder {
    config: RegistryConfig,
}

impl RegistryBuilder {
    pub fn new() -> Self { Self { config: RegistryConfig::default() } }
    pub fn build(self) -> RegistryConfig { self.config }
}

impl Default for RegistryBuilder {
    fn default() -> Self { Self::new() }
}
