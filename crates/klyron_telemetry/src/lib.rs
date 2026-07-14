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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryTransport {
  Grpc,
  Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
  pub service_name: String,
  pub service_version: String,
  pub otlp_endpoint: String,
  pub transport: TelemetryTransport,
  pub sampling_rate: f64,
  pub batch_size: u32,
  pub export_interval_ms: u64,
  pub enabled: bool,
}

impl Default for TelemetryConfig {
  fn default() -> Self {
    TelemetryConfig {
      service_name: "klyron".into(),
      service_version: "0.1.0".into(),
      otlp_endpoint: "http://localhost:4317".into(),
      transport: TelemetryTransport::Grpc,
      sampling_rate: 1.0,
      batch_size: 100,
      export_interval_ms: 5000,
      enabled: true,
    }
  }
}

pub struct TelemetryManager {
  config: TelemetryConfig,
  trace_count: AtomicU64,
}

impl TelemetryManager {
  pub fn new() -> Self {
    TelemetryManager {
      config: TelemetryConfig::default(),
      trace_count: AtomicU64::new(0),
    }
  }

  pub fn with_config(config: TelemetryConfig) -> Self {
    TelemetryManager {
      config,
      trace_count: AtomicU64::new(0),
    }
  }

  pub fn init(&mut self) -> Result<()> {
    if !self.config.enabled {
      return Ok(());
    }

    let svc_name: &'static str = Box::leak(self.config.service_name.clone().into_boxed_str());
    let svc_ver: &'static str = Box::leak(self.config.service_version.clone().into_boxed_str());

    let resource = Resource::new(vec![
      KeyValue::new("service.name", svc_name),
      KeyValue::new("service.version", svc_ver),
    ]);

    let exporter = match self.config.transport {
      TelemetryTransport::Grpc => opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(self.config.otlp_endpoint.clone())
        .build()
        .context("failed to build gRPC OTLP span exporter")?,
      TelemetryTransport::Http => opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(self.config.otlp_endpoint.clone())
        .build()
        .context("failed to build HTTP OTLP span exporter")?,
    };

    let tracer_provider = sdktrace::TracerProvider::builder()
      .with_batch_exporter(exporter, runtime::Tokio)
      .with_resource(resource.clone())
      .with_sampler(opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(self.config.sampling_rate))
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

    Ok(())
  }

  pub fn start_span(&self, span_name: &str) -> Option<opentelemetry::global::BoxedSpan> {
    let count = self.trace_count.fetch_add(1, Ordering::SeqCst);
    if count as f64 >= self.config.sampling_rate * 100.0 {
      return None;
    }
    let svc_name: &'static str = Box::leak(self.config.service_name.clone().into_boxed_str());
    let tracer = global::tracer(svc_name);
    Some(tracer.start(span_name.to_string()))
  }

  pub fn record_count(&self, name: &'static str, value: u64, attributes: Vec<KeyValue>) {
    let meter = global::meter(name);
    let counter = meter.u64_counter(name).with_description("").build();
    counter.add(value, &attributes);
  }

  pub fn record_gauge(&self, name: &'static str, value: u64, attributes: Vec<KeyValue>) {
    let meter = global::meter(name);
    let gauge = meter.u64_gauge(name).with_description("").build();
    gauge.record(value, &attributes);
  }

  pub fn record_histogram(&self, name: &'static str, value: f64, attributes: Vec<KeyValue>) {
    let meter = global::meter(name);
    let histogram = meter.f64_histogram(name).with_description("").build();
    histogram.record(value, &attributes);
  }

  pub fn shutdown(&self) {
    global::shutdown_tracer_provider();
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
  fn test_telemetry_config_default() {
    let config = TelemetryConfig::default();
    assert_eq!(config.service_name, "klyron");
    assert_eq!(config.sampling_rate, 1.0);
    assert!(config.enabled);
  }

  #[test]
  fn test_telemetry_manager_new() {
    let mgr = TelemetryManager::new();
    assert_eq!(mgr.trace_count.load(Ordering::SeqCst), 0);
  }

  #[test]
  fn test_telemetry_manager_disabled() {
    let config = TelemetryConfig {
      enabled: false,
      ..Default::default()
    };
    let mut mgr = TelemetryManager::with_config(config);
    assert!(mgr.init().is_ok());
    assert_eq!(mgr.trace_count.load(Ordering::SeqCst), 0);
  }

  #[test]
  fn test_start_span_without_init() {
    let mgr = TelemetryManager::new();
    let span = mgr.start_span("test");
    assert!(span.is_some());
  }

  #[test]
  fn test_telemetry_config_serialization() {
    let config = TelemetryConfig {
      service_name: "test-svc".into(),
      service_version: "1.0.0".into(),
      otlp_endpoint: "http://localhost:4318".into(),
      transport: TelemetryTransport::Http,
      sampling_rate: 0.5,
      batch_size: 50,
      export_interval_ms: 10000,
      enabled: true,
    };
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: TelemetryConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.service_name, "test-svc");
    assert_eq!(deserialized.transport, TelemetryTransport::Http);
    assert_eq!(deserialized.sampling_rate, 0.5);
  }
}
