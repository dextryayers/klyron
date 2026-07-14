#[derive(Debug, Clone)]
pub struct TranspilerConfig {
    pub enabled: bool,
    pub verbose: bool,
}

impl Default for TranspilerConfig {
    fn default() -> Self { Self { enabled: true, verbose: false } }
}
