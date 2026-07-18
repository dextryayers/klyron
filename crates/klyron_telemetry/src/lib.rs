pub mod export;
pub mod metrics;

use std::sync::Arc;

use anyhow::Result;

pub use export::{ExportConfig, TelemetryExporter, Transport};
pub use metrics::{HistogramSummary, MetricSnapshot, MetricsCollector};

pub struct TelemetryManager {
    collector: Arc<MetricsCollector>,
    exporter: TelemetryExporter,
    #[allow(dead_code)]
    service_name: String,
}

impl TelemetryManager {
    pub fn new() -> Self {
        Self {
            collector: Arc::new(MetricsCollector::new()),
            exporter: TelemetryExporter::new(),
            service_name: "klyron".to_string(),
        }
    }

    pub fn with_config(config: ExportConfig) -> Self {
        let service_name = config.service_name.clone();
        Self {
            collector: Arc::new(MetricsCollector::new()),
            exporter: TelemetryExporter::with_config(config),
            service_name,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.exporter.init()
    }

    pub fn collector(&self) -> &MetricsCollector {
        &self.collector
    }

    pub fn exporter(&self) -> &TelemetryExporter {
        &self.exporter
    }

    pub fn inc_counter(&self, name: &str, value: u64) {
        self.collector.inc_counter(name, value);
    }

    pub fn set_gauge(&self, name: &str, value: u64) {
        self.collector.set_gauge(name, value);
    }

    pub fn record_histogram(&self, name: &str, value: f64) {
        self.collector.record_histogram(name, value);
    }

    pub fn record_count(&self, name: &str, value: u64, attributes: Vec<opentelemetry::KeyValue>) {
        self.exporter.record_count(name, value, attributes);
    }

    pub fn record_gauge(&self, name: &str, value: u64, attributes: Vec<opentelemetry::KeyValue>) {
        self.exporter.record_gauge(name, value, attributes);
    }

    pub fn record_histogram_export(&self, name: &str, value: f64, attributes: Vec<opentelemetry::KeyValue>) {
        self.exporter.record_histogram(name, value, attributes);
    }

    pub fn start_span(&self, span_name: &str) -> Option<opentelemetry::global::BoxedSpan> {
        self.exporter.start_span(span_name)
    }

    pub fn flush(&self) -> Result<()> {
        let snapshot = self.collector.snapshot();
        self.exporter.export_snapshot(snapshot)?;
        self.collector.reset();
        Ok(())
    }

    pub fn shutdown(&self) {
        self.exporter.shutdown();
    }

    pub fn snapshot(&self) -> MetricSnapshot {
        self.collector.snapshot()
    }
}

impl Default for TelemetryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_manager_new() {
        let mgr = TelemetryManager::new();
        assert_eq!(mgr.collector.get_counter("test"), 0);
    }

    #[test]
    fn test_telemetry_counters() {
        let mgr = TelemetryManager::new();
        mgr.inc_counter("hits", 42);
        assert_eq!(mgr.collector.get_counter("hits"), 42);
    }

    #[test]
    fn test_telemetry_histogram() {
        let mgr = TelemetryManager::new();
        mgr.record_histogram("latency", 10.0);
        let snap = mgr.snapshot();
        assert_eq!(snap.histograms["latency"].count, 1);
    }

    #[test]
    fn test_telemetry_flush() {
        let mut mgr = TelemetryManager::with_config(ExportConfig {
            enabled: false,
            ..Default::default()
        });
        mgr.init().unwrap();
        mgr.inc_counter("flush_test", 7);
        assert!(mgr.flush().is_ok());
        assert_eq!(mgr.collector.get_counter("flush_test"), 0);
    }

    #[test]
    fn test_telemetry_manager_disabled() {
        let config = ExportConfig {
            enabled: false,
            ..Default::default()
        };
        let mut mgr = TelemetryManager::with_config(config);
        assert!(mgr.init().is_ok());
    }
}
