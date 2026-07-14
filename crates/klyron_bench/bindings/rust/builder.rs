use crate::config::BenchConfig;

pub struct BenchBuilder {
    config: BenchConfig,
}

impl BenchBuilder {
    pub fn new() -> Self { Self { config: BenchConfig::default() } }
    pub fn build(self) -> BenchConfig { self.config }
}

impl Default for BenchBuilder {
    fn default() -> Self { Self::new() }
}
