pub struct BenchClient {
    config: crate::config::BenchConfig,
}

impl BenchClient {
    pub fn new() -> Self { Self { config: crate::config::BenchConfig::default() } }
    pub fn config(&self) -> &crate::config::BenchConfig { &self.config }
    pub fn version(&self) -> &'static str { "1.0.0" }
}
