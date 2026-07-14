#[derive(Debug, Clone)]
pub struct BundlerConfig {
    pub enabled: bool,
    pub verbose: bool,
}

impl Default for BundlerConfig {
    fn default() -> Self { Self { enabled: true, verbose: false } }
}
