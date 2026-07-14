use crate::config::TestConfig;

pub struct TestBuilder {
    config: TestConfig,
}

impl TestBuilder {
    pub fn new() -> Self { Self { config: TestConfig::default() } }
    pub fn build(self) -> TestConfig { self.config }
}

impl Default for TestBuilder {
    fn default() -> Self { Self::new() }
}
