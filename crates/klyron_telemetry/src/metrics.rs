use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
    pub histograms: HashMap<String, HistogramSummary>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramSummary {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

pub struct MetricsCollector {
    counters: Mutex<HashMap<String, AtomicU64>>,
    gauges: Mutex<HashMap<String, AtomicU64>>,
    histograms: Mutex<HashMap<String, HistogramBuffer>>,
    labels: Mutex<HashMap<String, String>>,
    start_time: Instant,
}

struct HistogramBuffer {
    values: Vec<f64>,
    sum: f64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: Mutex::new(HashMap::new()),
            gauges: Mutex::new(HashMap::new()),
            histograms: Mutex::new(HashMap::new()),
            labels: Mutex::new(HashMap::new()),
            start_time: Instant::now(),
        }
    }

    pub fn inc_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.lock().unwrap();
        let counter = counters.entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(value, Ordering::SeqCst);
    }

    pub fn set_gauge(&self, name: &str, value: u64) {
        let mut gauges = self.gauges.lock().unwrap();
        let gauge = gauges.entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        gauge.store(value, Ordering::SeqCst);
    }

    pub fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.lock().unwrap();
        let buf = histograms.entry(name.to_string())
            .or_insert_with(|| HistogramBuffer {
                values: Vec::new(),
                sum: 0.0,
            });
        buf.values.push(value);
        buf.sum += value;
    }

    pub fn set_label(&self, key: &str, value: &str) {
        self.labels.lock().unwrap().insert(key.to_string(), value.to_string());
    }

    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.lock().unwrap()
            .get(name)
            .map(|c| c.load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    pub fn get_gauge(&self, name: &str) -> u64 {
        self.gauges.lock().unwrap()
            .get(name)
            .map(|c| c.load(Ordering::SeqCst))
            .unwrap_or(0)
    }

    pub fn snapshot(&self) -> MetricSnapshot {
        let counters: HashMap<String, u64> = self.counters.lock().unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::SeqCst)))
            .collect();

        let gauges: HashMap<String, f64> = self.gauges.lock().unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::SeqCst) as f64))
            .collect();

        let histograms: HashMap<String, HistogramSummary> = self.histograms.lock().unwrap()
            .iter()
            .map(|(k, buf)| {
                let count = buf.values.len() as u64;
                let avg = if count > 0 { buf.sum / count as f64 } else { 0.0 };
                let min = buf.values.iter().cloned().fold(f64::MAX, f64::min);
                let max = buf.values.iter().cloned().fold(f64::MIN, f64::max);
                (k.clone(), HistogramSummary {
                    count,
                    sum: buf.sum,
                    min: if count > 0 { min } else { 0.0 },
                    max: if count > 0 { max } else { 0.0 },
                    avg,
                })
            })
            .collect();

        MetricSnapshot {
            counters,
            gauges,
            histograms,
            timestamp: self.start_time.elapsed().as_secs(),
        }
    }

    pub fn reset(&self) {
        self.counters.lock().unwrap().clear();
        self.gauges.lock().unwrap().clear();
        self.histograms.lock().unwrap().clear();
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let mc = MetricsCollector::new();
        assert_eq!(mc.get_counter("requests"), 0);
        mc.inc_counter("requests", 1);
        mc.inc_counter("requests", 2);
        assert_eq!(mc.get_counter("requests"), 3);
    }

    #[test]
    fn test_gauge() {
        let mc = MetricsCollector::new();
        mc.set_gauge("connections", 10);
        assert_eq!(mc.get_gauge("connections"), 10);
        mc.set_gauge("connections", 5);
        assert_eq!(mc.get_gauge("connections"), 5);
    }

    #[test]
    fn test_histogram() {
        let mc = MetricsCollector::new();
        mc.record_histogram("latency", 1.0);
        mc.record_histogram("latency", 3.0);
        mc.record_histogram("latency", 5.0);
        let snapshot = mc.snapshot();
        let hist = snapshot.histograms.get("latency").unwrap();
        assert_eq!(hist.count, 3);
        assert!((hist.avg - 3.0).abs() < 0.001);
        assert!((hist.min - 1.0).abs() < 0.001);
        assert!((hist.max - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_snapshot() {
        let mc = MetricsCollector::new();
        mc.inc_counter("hits", 42);
        mc.set_gauge("temp", 25);
        mc.record_histogram("response_time", 0.5);
        let snap = mc.snapshot();
        assert_eq!(snap.counters["hits"], 42);
        assert!((snap.gauges["temp"] - 25.0).abs() < 0.001);
        assert_eq!(snap.histograms["response_time"].count, 1);
    }

    #[test]
    fn test_labels() {
        let mc = MetricsCollector::new();
        mc.set_label("service", "klyron");
        assert!(mc.labels.lock().unwrap().contains_key("service"));
    }

    #[test]
    fn test_reset() {
        let mc = MetricsCollector::new();
        mc.inc_counter("hits", 1);
        mc.reset();
        assert_eq!(mc.get_counter("hits"), 0);
    }
}
