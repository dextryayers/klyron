//! Server for klyron_telemetry
pub struct TelemetryServer;

impl TelemetryServer {
    pub fn new() -> Self { Self }
    pub fn serve(&self, _addr: &str) {}
}
