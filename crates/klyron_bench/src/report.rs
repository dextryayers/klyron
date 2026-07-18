use crate::BenchCategory;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    pub category: BenchCategory,
    pub label: String,
    pub iterations: u64,
    pub warmup_iterations: u64,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub median_time: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub stddev: f64,
    pub ops_per_sec: f64,
    pub samples: Vec<f64>,
}

impl BenchResult {
    pub fn formatted_report(&self) -> String {
        format!(
            "{:30} | {:>10} iters | {:>12} avg | {:>12} min | {:>12} max | {:>10.1} ops/s | p50 {:>10} | p95 {:>10} | p99 {:>10}",
            self.label,
            self.iterations,
            fmt_duration(self.avg_time),
            fmt_duration(self.min_time),
            fmt_duration(self.max_time),
            self.ops_per_sec,
            fmt_duration(self.p50),
            fmt_duration(self.p95),
            fmt_duration(self.p99),
        )
    }

    pub fn relative_stddev(&self) -> f64 {
        if self.avg_time.as_nanos() > 0 {
            self.stddev / self.avg_time.as_nanos() as f64
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchReport {
    pub results: Vec<BenchResult>,
    pub baseline: Option<Box<BenchReport>>,
    pub comparisons: Vec<super::compare::BenchComparison>,
    pub timestamp: String,
}

impl BenchReport {
    pub fn new(results: Vec<BenchResult>) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();
        Self {
            results,
            baseline: None,
            comparisons: Vec::new(),
            timestamp,
        }
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn save_json(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let json = self.to_json_pretty()?;
        std::fs::write(path, json).map_err(anyhow::Error::from)
    }

    pub fn formatted(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Benchmark Report ({})\n", self.timestamp));
        out.push_str(&format!(
            "{:-<120}\n",
            ""
        ));
        out.push_str(&format!(
            "{:30} | {:>10} | {:>12} | {:>12} | {:>12} | {:>10} | {:>10} | {:>10} | {:>10}\n",
            "Label", "Iterations", "Avg", "Min", "Max", "Ops/s", "p50", "p95", "p99"
        ));
        out.push_str(&format!("{:-<120}\n", ""));
        for r in &self.results {
            out.push_str(&r.formatted_report());
            out.push('\n');
        }
        out
    }

    pub fn print(&self) {
        println!("{}", self.formatted());
    }
}

fn fmt_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs >= 1.0 {
        format!("{:.3}s", secs)
    } else if secs >= 0.001 {
        format!("{:.3}ms", secs * 1000.0)
    } else if secs >= 0.000_001 {
        format!("{:.3}µs", secs * 1_000_000.0)
    } else {
        format!("{:.1}ns", secs * 1_000_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_report_json() {
        let report = BenchReport {
            results: vec![BenchResult {
                category: BenchCategory::Micro,
                label: "test".into(),
                iterations: 100,
                warmup_iterations: 10,
                total_time: Duration::from_secs(1),
                avg_time: Duration::from_millis(10),
                min_time: Duration::from_millis(1),
                max_time: Duration::from_millis(100),
                median_time: Duration::from_millis(10),
                p50: Duration::from_millis(10),
                p95: Duration::from_millis(50),
                p99: Duration::from_millis(90),
                stddev: 0.5,
                ops_per_sec: 100.0,
                samples: vec![1.0, 2.0, 3.0],
            }],
            baseline: None,
            comparisons: vec![],
            timestamp: "2024-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&report).expect("Serialization failed");
        let deserialized: BenchReport =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.results.len(), 1);
        assert_eq!(deserialized.results[0].label, "test");
        assert!(deserialized.results[0].samples.len() == 3);
    }

    #[test]
    fn test_fmt_duration() {
        let d = Duration::from_nanos(500);
        assert!(fmt_duration(d).contains("ns"));
        let d = Duration::from_micros(50);
        assert!(fmt_duration(d).contains("µs"));
        let d = Duration::from_millis(10);
        assert!(fmt_duration(d).contains("ms"));
        let d = Duration::from_secs(2);
        assert!(fmt_duration(d).contains("s"));
    }
}
