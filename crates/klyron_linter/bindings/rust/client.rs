pub struct LinterClient {
    config: crate::config::LinterConfig,
}

impl LinterClient {
    pub fn new() -> Self { Self { config: crate::config::LinterConfig::default() } }
    pub fn config(&self) -> &crate::config::LinterConfig { &self.config }
    pub fn version(&self) -> &'static str { "1.0.0" }
}
