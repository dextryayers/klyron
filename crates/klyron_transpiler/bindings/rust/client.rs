pub struct TranspilerClient {
    config: crate::config::TranspilerConfig,
}

impl TranspilerClient {
    pub fn new() -> Self { Self { config: crate::config::TranspilerConfig::default() } }
    pub fn config(&self) -> &crate::config::TranspilerConfig { &self.config }
    pub fn version(&self) -> &'static str { "1.0.0" }
}
