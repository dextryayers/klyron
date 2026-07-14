#[derive(Debug, Clone)]
pub struct TestConfig {
    pub enabled: bool,
    pub verbose: bool,
}

impl Default for TestConfig {
    fn default() -> Self { Self { enabled: true, verbose: false } }
}
