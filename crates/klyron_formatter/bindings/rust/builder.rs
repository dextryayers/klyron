use crate::config::FormatterConfig;

pub struct FormatterBuilder {
    config: FormatterConfig,
}

impl FormatterBuilder {
    pub fn new() -> Self { Self { config: FormatterConfig::default() } }
    pub fn build(self) -> FormatterConfig { self.config }
}

impl Default for FormatterBuilder {
    fn default() -> Self { Self::new() }
}
