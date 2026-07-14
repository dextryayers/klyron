#[derive(Debug, Clone)]
pub struct BenchConfig {
    pub enabled: bool,
    pub verbose: bool,
}

impl Default for BenchConfig {
    fn default() -> Self { Self { enabled: true, verbose: false } }
}
