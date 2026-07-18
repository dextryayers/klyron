use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::{Context, Result};
use opentelemetry::{
    global,
    trace::Tracer,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics as sdk_metrics,
    runtime,
    trace as sdktrace,
    Resource,
};
use serde::{Deserialize, Serialize};

use crate::metrics::MetricSnapshot;

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    Grpc,
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub service_name: String,
    pub service_version: String,
    pub otlp_endpoint: String,
    pub transport: Transport,
    pub sampling_rate: f64,
    pub batch_size: u32,
    pub export_interval_ms: u64,
    pub enabled: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            service_name: "klyron".into(),
            service_version: "0.1.0".into(),
            otlp_endpoint: "http://localhost:4317".into(),
            transport: Transport::Grpc,
            sampling_rate: 1.0,
            batch_size: 100,
            export_interval_ms: 5000,
            enabled: true,
        }
    }
}

pub struct TelemetryExporter {
    config: ExportConfig,
    trace_count: AtomicU64,
    initialized: bool,
}

impl TelemetryExporter {
    pub fn new() -> Self {
        Self {
            config: ExportConfig::default(),
            trace_count: AtomicU64::new(0),
            initialized: false,
        }
    }

    pub fn with_config(config: ExportConfig) -> Self {
        Self {
            config,
            trace_count: AtomicU64::new(0),
            initialized: false,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        if !self.config.enabled || self.initialized {
            return Ok(());
        }

        let svc_name: &'static str = Box::leak(self.config.service_name.clone().into_boxed_str());
        let svc_ver: &'static str = Box::leak(self.config.service_version.clone().into_boxed_str());

        let resource = Resource::new(vec![
            KeyValue::new("service.name", svc_name),
            KeyValue::new("service.version", svc_ver),
        ]);

        let exporter = match self.config.transport {
            Transport::Grpc => opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(self.config.otlp_endpoint.clone())
                .build()
                .context("failed to build gRPC OTLP span exporter")?,
            Transport::Http => opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .with_endpoint(self.config.otlp_endpoint.clone())
                .build()
                .context("failed to build HTTP OTLP span exporter")?,
        };

        let tracer_provider = sdktrace::TracerProvider::builder()
            .with_batch_exporter(exporter, runtime::Tokio)
            .with_resource(resource.clone())
            .with_sampler(
                opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(self.config.sampling_rate),
            )
            .build();
        global::set_tracer_provider(tracer_provider);

        let meter_exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .build()
            .context("failed to build OTLP metric exporter")?;

        let meter_provider = sdk_metrics::SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(
                sdk_metrics::PeriodicReader::builder(meter_exporter, runtime::Tokio)
                    .with_interval(Duration::from_millis(self.config.export_interval_ms))
                    .build(),
            )
            .build();
        global::set_meter_provider(meter_provider);

        self.initialized = true;
        Ok(())
    }

    pub fn start_span(&self, span_name: &str) -> Option<opentelemetry::global::BoxedSpan> {
        let count = self.trace_count.fetch_add(1, Ordering::SeqCst);
        if count as f64 >= self.config.sampling_rate * 100.0 {
            return None;
        }
        let svc_name = leak(&self.config.service_name);
        let tracer = global::tracer(svc_name);
        Some(tracer.start(span_name.to_string()))
    }

    pub fn record_count(&self, name: &str, value: u64, attributes: Vec<KeyValue>) {
        let name = leak(name);
        let meter = global::meter(name);
        let counter = meter.u64_counter(name).build();
        counter.add(value, &attributes);
    }

    pub fn record_gauge(&self, name: &str, value: u64, attributes: Vec<KeyValue>) {
        let name = leak(name);
        let meter = global::meter(name);
        let gauge = meter.u64_gauge(name).build();
        gauge.record(value, &attributes);
    }

    pub fn record_histogram(&self, name: &str, value: f64, attributes: Vec<KeyValue>) {
        let name = leak(name);
        let meter = global::meter(name);
        let histogram = meter.f64_histogram(name).build();
        histogram.record(value, &attributes);
    }

    pub fn export_snapshot(&self, snapshot: MetricSnapshot) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        let counters = snapshot.counters;
        let gauges = snapshot.gauges;
        let histograms = snapshot.histograms;
        for (name, value) in &counters {
            self.record_count(name, *value, vec![]);
        }
        for (name, value) in &gauges {
            self.record_gauge(name, *value as u64, vec![]);
        }
        for (name, hist) in &histograms {
            self.record_histogram(name, hist.avg, vec![]);
        }
        Ok(())
    }

    pub fn shutdown(&self) {
        global::shutdown_tracer_provider();
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for TelemetryExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        assert_eq!(config.service_name, "klyron");
        assert!(config.enabled);
    }

    #[test]
    fn test_exporter_disabled() {
        let config = ExportConfig {
            enabled: false,
            ..Default::default()
        };
        let mut exporter = TelemetryExporter::with_config(config);
        assert!(exporter.init().is_ok());
        assert!(!exporter.is_initialized());
    }

    #[test]
    fn test_start_span_without_init() {
        let exporter = TelemetryExporter::new();
        let span = exporter.start_span("test");
        assert!(span.is_some());
    }

    #[test]
    fn test_export_config_serialization() {
        let config = ExportConfig {
            service_name: "test-svc".into(),
            service_version: "1.0.0".into(),
            otlp_endpoint: "http://localhost:4318".into(),
            transport: Transport::Http,
            sampling_rate: 0.5,
            batch_size: 50,
            export_interval_ms: 10000,
            enabled: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ExportConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.service_name, "test-svc");
        assert_eq!(deserialized.transport, Transport::Http);
    }

    #[test]
    fn test_export_snapshot_empty() {
        let exporter = TelemetryExporter::new();
        let snapshot = MetricSnapshot::default();
        assert!(exporter.export_snapshot(snapshot).is_ok());
    }
}
