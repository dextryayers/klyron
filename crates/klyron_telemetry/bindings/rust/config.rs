//! Config for klyron_telemetry
use crate::types::TelemetryConfig;

pub fn load_config() -> TelemetryConfig {
    TelemetryConfig::default()
}
