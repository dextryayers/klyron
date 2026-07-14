pub struct TestClient {
    config: crate::config::TestConfig,
}

impl TestClient {
    pub fn new() -> Self { Self { config: crate::config::TestConfig::default() } }
    pub fn config(&self) -> &crate::config::TestConfig { &self.config }
    pub fn version(&self) -> &'static str { "1.0.0" }
}
