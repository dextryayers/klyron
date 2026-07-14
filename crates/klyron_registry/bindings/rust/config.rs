#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub enabled: bool,
    pub verbose: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self { Self { enabled: true, verbose: false } }
}
