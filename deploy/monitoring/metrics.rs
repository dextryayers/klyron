use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Counter {
    name: String,
    help: String,
    value: u64,
    labels: HashMap<String, String>,
}

impl Counter {
    pub fn new(name: &str, help: &str) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            value: 0,
            labels: HashMap::new(),
        }
    }

    pub fn with_labels(name: &str, help: &str, labels: HashMap<String, String>) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            value: 0,
            labels,
        }
    }

    pub fn inc(&mut self) {
        self.value += 1;
    }

    pub fn add(&mut self, n: u64) {
        self.value += n;
    }

    pub fn value(&self) -> u64 {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct Gauge {
    name: String,
    help: String,
    value: f64,
    labels: HashMap<String, String>,
}

impl Gauge {
    pub fn new(name: &str, help: &str) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            value: 0.0,
            labels: HashMap::new(),
        }
    }

    pub fn set(&mut self, value: f64) {
        self.value = value;
    }

    pub fn inc(&mut self) {
        self.value += 1.0;
    }

    pub fn dec(&mut self) {
        self.value -= 1.0;
    }

    pub fn add(&mut self, n: f64) {
        self.value += n;
    }

    pub fn sub(&mut self, n: f64) {
        self.value -= n;
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    help: String,
    buckets: Vec<f64>,
    counts: Vec<u64>,
    sum: f64,
    count: u64,
    labels: HashMap<String, String>,
}

impl Histogram {
    pub fn new(name: &str, help: &str, buckets: Vec<f64>) -> Self {
        let n = buckets.len();
        Self {
            name: name.to_string(),
            help: help.to_string(),
            buckets,
            counts: vec![0; n + 1],
            sum: 0.0,
            count: 0,
            labels: HashMap::new(),
        }
    }

    pub fn observe(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        for (i, bucket) in self.buckets.iter().enumerate() {
            if value <= *bucket {
                self.counts[i] += 1;
            }
        }
        self.counts[self.buckets.len()] = self.count;
    }

    pub fn observe_duration(&mut self, d: Duration) {
        self.observe(d.as_secs_f64());
    }
}

#[derive(Debug, Clone)]
pub struct MetricsRegistry {
    counters: HashMap<String, Counter>,
    gauges: HashMap<String, Gauge>,
    histograms: HashMap<String, Histogram>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }

    pub fn register_counter(&mut self, name: &str, help: &str) {
        self.counters.entry(name.to_string()).or_insert_with(|| Counter::new(name, help));
    }

    pub fn register_gauge(&mut self, name: &str, help: &str) {
        self.gauges.entry(name.to_string()).or_insert_with(|| Gauge::new(name, help));
    }

    pub fn register_histogram(&mut self, name: &str, help: &str, buckets: Vec<f64>) {
        self.histograms.entry(name.to_string()).or_insert_with(|| Histogram::new(name, help, buckets));
    }

    pub fn get_counter(&self, name: &str) -> Option<&Counter> {
        self.counters.get(name)
    }

    pub fn get_counter_mut(&mut self, name: &str) -> Option<&mut Counter> {
        self.counters.get_mut(name)
    }

    pub fn get_gauge(&self, name: &str) -> Option<&Gauge> {
        self.gauges.get(name)
    }

    pub fn get_gauge_mut(&mut self, name: &str) -> Option<&mut Gauge> {
        self.gauges.get_mut(name)
    }

    pub fn get_histogram(&self, name: &str) -> Option<&Histogram> {
        self.histograms.get(name)
    }

    pub fn get_histogram_mut(&mut self, name: &str) -> Option<&mut Histogram> {
        self.histograms.get_mut(name)
    }

    pub fn inc_counter(&mut self, name: &str) {
        if let Some(c) = self.counters.get_mut(name) {
            c.inc();
        }
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) {
        if let Some(g) = self.gauges.get_mut(name) {
            g.set(value);
        }
    }

    pub fn observe_histogram(&mut self, name: &str, value: f64) {
        if let Some(h) = self.histograms.get_mut(name) {
            h.observe(value);
        }
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub counters: Vec<MetricSample>,
    pub gauges: Vec<MetricSample>,
    pub histograms: Vec<HistogramSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub name: String,
    pub help: String,
    pub value: String,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSample {
    pub name: String,
    pub help: String,
    pub sum: f64,
    pub count: u64,
    pub buckets: Vec<(f64, u64)>,
    pub labels: HashMap<String, String>,
}

pub fn collect_metrics(registry: &MetricsRegistry) -> MetricsSnapshot {
    let now = Utc::now();
    let counters = registry
        .counters
        .values()
        .map(|c| MetricSample {
            name: c.name.clone(),
            help: c.help.clone(),
            value: c.value.to_string(),
            labels: c.labels.clone(),
        })
        .collect();
    let gauges = registry
        .gauges
        .values()
        .map(|g| MetricSample {
            name: g.name.clone(),
            help: g.help.clone(),
            value: g.value.to_string(),
            labels: g.labels.clone(),
        })
        .collect();
    let histograms = registry
        .histograms
        .values()
        .map(|h| {
            let buckets: Vec<(f64, u64)> = h
                .buckets
                .iter()
                .enumerate()
                .map(|(i, b)| (*b, h.counts[i]))
                .collect();
            HistogramSample {
                name: h.name.clone(),
                help: h.help.clone(),
                sum: h.sum,
                count: h.count,
                buckets,
                labels: h.labels.clone(),
            }
        })
        .collect();
    MetricsSnapshot {
        timestamp: now,
        counters,
        gauges,
        histograms,
    }
}

pub struct PrometheusFormatter;

impl PrometheusFormatter {
    pub fn format(snapshot: &MetricsSnapshot) -> String {
        let mut out = String::new();
        out.push_str("# HELP klyron_metrics Metrics snapshot\n");
        out.push_str("# TYPE klyron_metrics untyped\n");
        out.push_str(&format!("klyron_metrics_timestamp {}\n", snapshot.timestamp.timestamp()));
        for c in &snapshot.counters {
            out.push_str(&format!("# HELP {} {}\n", c.name, c.help));
            out.push_str(&format!("# TYPE {} counter\n", c.name));
            out.push_str(&format!("{} {}\n", c.name, c.value));
        }
        for g in &snapshot.gauges {
            out.push_str(&format!("# HELP {} {}\n", g.name, g.help));
            out.push_str(&format!("# TYPE {} gauge\n", g.name));
            out.push_str(&format!("{} {}\n", g.name, g.value));
        }
        for h in &snapshot.histograms {
            out.push_str(&format!("# HELP {} {}\n", h.name, h.help));
            out.push_str(&format!("# TYPE {} histogram\n", h.name));
            out.push_str(&format!("{}_sum {}\n", h.name, h.sum));
            out.push_str(&format!("{}_count {}\n", h.name, h.count));
            for (bucket, count) in &h.buckets {
                out.push_str(&format!("{}_bucket{{le=\"{}\"}} {}\n", h.name, bucket, count));
            }
        }
        out
    }
}

pub struct SystemMetrics;

impl SystemMetrics {
    pub fn collect() -> MetricsSnapshot {
        let mut registry = MetricsRegistry::new();
        if let Ok(info) = sys_info::cpu_num() {
            registry.set_gauge("system_cpu_cores", info as f64);
        }
        if let Ok(load) = sys_info::loadavg() {
            registry.set_gauge("system_load_1m", load.one as f64);
            registry.set_gauge("system_load_5m", load.five as f64);
            registry.set_gauge("system_load_15m", load.fifteen as f64);
        }
        if let Ok(mem) = sys_info::mem_info() {
            registry.set_gauge("system_memory_total_bytes", (mem.total * 1024) as f64);
            registry.set_gauge("system_memory_free_bytes", (mem.free * 1024) as f64);
            registry.set_gauge("system_memory_available_bytes", (mem.avail * 1024) as f64);
        }
        if let Ok(disk) = sys_info::disk_info() {
            registry.set_gauge("system_disk_total_bytes", disk.total as f64);
            registry.set_gauge("system_disk_free_bytes", disk.free as f64);
        }
        collect_metrics(&registry)
    }
}

pub struct RuntimeMetrics;

impl RuntimeMetrics {
    pub fn collect() -> MetricsSnapshot {
        let mut registry = MetricsRegistry::new();
        registry.set_gauge("runtime_active_requests", 0.0);
        registry.set_gauge("runtime_memory_heap_bytes", {
            let _prof = &();
            0.0
        });
        collect_metrics(&registry)
    }
}

pub struct BusinessMetrics {
    registry: MetricsRegistry,
}

impl BusinessMetrics {
    pub fn new() -> Self {
        let mut registry = MetricsRegistry::new();
        registry.register_counter("http_requests_total", "Total HTTP requests");
        registry.register_counter("http_errors_total", "Total HTTP errors");
        registry.register_histogram("http_request_duration_seconds", "HTTP request latency", vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]);
        Self { registry }
    }

    pub fn record_request(&mut self, duration: Duration, status: u16) {
        self.registry.inc_counter("http_requests_total");
        if status >= 500 {
            self.registry.inc_counter("http_errors_total");
        }
        self.registry.observe_histogram("http_request_duration_seconds", duration.as_secs_f64());
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        collect_metrics(&self.registry)
    }

    pub fn latency_percentiles(&self) -> (f64, f64, f64) {
        (0.0, 0.0, 0.0)
    }
}

impl Default for BusinessMetrics {
    fn default() -> Self {
        Self::new()
    }
}
