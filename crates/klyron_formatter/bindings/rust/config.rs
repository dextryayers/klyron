#[derive(Debug, Clone)]
pub struct FormatterConfig {
    pub enabled: bool,
    pub verbose: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self { Self { enabled: true, verbose: false } }
}
