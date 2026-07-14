#[derive(Debug, Clone)]
pub struct LinterConfig {
    pub enabled: bool,
    pub verbose: bool,
}

impl Default for LinterConfig {
    fn default() -> Self { Self { enabled: true, verbose: false } }
}
