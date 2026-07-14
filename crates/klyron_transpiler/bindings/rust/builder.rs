use crate::config::TranspilerConfig;

pub struct TranspilerBuilder {
    config: TranspilerConfig,
}

impl TranspilerBuilder {
    pub fn new() -> Self { Self { config: TranspilerConfig::default() } }
    pub fn build(self) -> TranspilerConfig { self.config }
}

impl Default for TranspilerBuilder {
    fn default() -> Self { Self::new() }
}
