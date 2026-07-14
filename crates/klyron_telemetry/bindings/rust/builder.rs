//! Builder for klyron_telemetry
use crate::types::TelemetryConfig;

pub struct TelemetryBuilder {
    config: TelemetryConfig,
}

impl TelemetryBuilder {
    pub fn new() -> Self {
        Self { config: TelemetryConfig::default() }
    }
    pub fn with_version(mut self, version: &str) -> Self {
        self.config.version = version.to_string();
        self
    }
    pub fn build(self) -> TelemetryConfig {
        self.config
    }
}
