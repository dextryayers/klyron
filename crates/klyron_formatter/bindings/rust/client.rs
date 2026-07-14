pub struct FormatterClient {
    config: crate::config::FormatterConfig,
}

impl FormatterClient {
    pub fn new() -> Self { Self { config: crate::config::FormatterConfig::default() } }
    pub fn config(&self) -> &crate::config::FormatterConfig { &self.config }
    pub fn version(&self) -> &'static str { "1.0.0" }
}
