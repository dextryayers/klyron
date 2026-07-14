//! Client for klyron_telemetry
use crate::types::TelemetryConfig;

pub struct TelemetryClient {
    config: TelemetryConfig,
}

impl TelemetryClient {
    pub fn new(config: TelemetryConfig) -> Self {
        Self { config }
    }
    pub fn execute(&self) -> crate::errors::Result<()> {
        Ok(())
    }
}
