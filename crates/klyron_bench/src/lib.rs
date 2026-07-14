use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use statrs::statistics::Statistics;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchCategory {
  Runtime,
  Http,
  Memory,
  Startup,
  Micro,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchReport {
  pub results: Vec<BenchResult>,
  pub baseline: Option<Box<BenchReport>>,
  pub comparisons: Vec<BenchComparison>,
  pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchComparison {
  pub label: String,
  pub baseline_avg: Duration,
  pub current_avg: Duration,
  pub change_pct: f64,
  pub regression: bool,
}

#[derive(Debug, Clone)]
pub struct BenchConfig {
  pub warmup_iterations: u64,
  pub min_iterations: u64,
  pub min_duration: Duration,
  pub baseline_path: Option<std::path::PathBuf>,
  pub json_output: Option<std::path::PathBuf>,
  pub criterion_mode: bool,
}

impl Default for BenchConfig {
  fn default() -> Self {
    BenchConfig {
      warmup_iterations: 100,
      min_iterations: 1000,
      min_duration: Duration::from_secs(2),
      baseline_path: None,
      json_output: None,
      criterion_mode: false,
    }
  }
}

pub struct BenchmarkRunner {
  config: BenchConfig,
}

impl BenchmarkRunner {
  pub fn new() -> Self {
    BenchmarkRunner {
      config: BenchConfig::default(),
    }
  }

  pub fn with_config(config: BenchConfig) -> Self {
    BenchmarkRunner { config }
  }

  pub fn run_all(dir: &Path) -> Result<BenchReport> {
    let mut results = Vec::new();
    let timestamp = chrono::Utc::now().to_rfc3339();

    if let Ok(r) = BenchmarkRunner::run_micro("dir_read", &mut || {
      let _ = std::fs::read_dir(dir);
    }) {
      results.push(r);
    }

    if let Ok(r) = Self::bench_memory() {
      results.push(r);
    }

    if let Ok(r) = Self::bench_startup(10) {
      results.push(r);
    }

    let report = BenchReport {
      results,
      baseline: None,
      comparisons: Vec::new(),
      timestamp,
    };

    Ok(report)
  }

  pub fn run_micro(label: &str, f: &mut dyn FnMut()) -> Result<BenchResult> {
    let config = BenchConfig::default();
    let runner = BenchmarkRunner::with_config(config);
    runner.run_micro_internal(label, f)
  }

  fn run_micro_internal(&self, label: &str, f: &mut dyn FnMut()) -> Result<BenchResult> {
    for _ in 0..self.config.warmup_iterations {
      (f)();
    }

    let mut samples = Vec::new();
    let mut min_time = Duration::MAX;
    let mut max_time = Duration::ZERO;
    let mut total_time = Duration::ZERO;
    let mut iterations: u64 = 0;

    let batch_size = 100u64;

    while total_time < self.config.min_duration || iterations < self.config.min_iterations {
      let batch_start = Instant::now();
      for _ in 0..batch_size {
        (f)();
      }
      let elapsed = batch_start.elapsed();

      let per_op = elapsed / batch_size as u32;
      samples.push(per_op.as_secs_f64() * 1_000_000_000.0);

      if per_op < min_time {
        min_time = per_op;
      }
      if per_op > max_time {
        max_time = per_op;
      }
      total_time += elapsed;
      iterations += batch_size;
    }

    let samples_ns: Vec<f64> = samples.clone();
    let mean = samples_ns.clone().mean();
    let stddev = samples_ns.clone().std_dev();
    let sorted = {
      let mut s = samples_ns.clone();
      s.sort_by(|a, b| a.partial_cmp(b).unwrap());
      s
    };

    let p50 = percentile(&sorted, 50);
    let p95 = percentile(&sorted, 95);
    let p99 = percentile(&sorted, 99);

    let avg_time = Duration::from_nanos(mean as u64);
    let median_time = Duration::from_nanos(p50 as u64);
    let ops_per_sec = if mean > 0.0 {
      1_000_000_000.0 / mean
    } else {
      0.0
    };

    Ok(BenchResult {
      category: BenchCategory::Micro,
      label: label.to_string(),
      iterations,
      warmup_iterations: self.config.warmup_iterations,
      total_time,
      avg_time,
      min_time,
      max_time,
      median_time,
      p50: Duration::from_nanos(p50 as u64),
      p95: Duration::from_nanos(p95 as u64),
      p99: Duration::from_nanos(p99 as u64),
      stddev,
      ops_per_sec,
      samples: samples_ns,
    })
  }

  pub fn bench_http(url: &str, connections: u32, duration: Duration) -> Result<BenchResult> {
    let (host, port, path) = Self::parse_url(url)?;
    let start = Instant::now();
    let mut samples = Vec::new();
    let mut count = 0u64;

    while start.elapsed() < duration {
      for _ in 0..connections {
        let op_start = Instant::now();
        let addr: std::net::SocketAddr = format!("{host}:{port}")
          .parse()
          .context("Invalid address")?;
        let mut stream = std::net::TcpStream::connect_timeout(
          &addr,
          Duration::from_secs(5),
        )?;
        let request = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
        use std::io::Write;
        stream.write_all(request.as_bytes())?;
        stream.flush()?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;
        let elapsed = op_start.elapsed();
        samples.push(elapsed.as_secs_f64() * 1_000_000_000.0);
        count += 1;
      }
    }

    let total_time = start.elapsed();
    let sorted = {
      let mut s = samples.clone();
      s.sort_by(|a, b| a.partial_cmp(b).unwrap());
      s
    };

    Ok(BenchResult {
      category: BenchCategory::Http,
      label: url.to_string(),
      iterations: count,
      warmup_iterations: 0,
      total_time,
      avg_time: total_time / count as u32,
      min_time: Duration::from_nanos(sorted.first().copied().unwrap_or(0.0) as u64),
      max_time: Duration::from_nanos(sorted.last().copied().unwrap_or(0.0) as u64),
      median_time: Duration::from_nanos(percentile(&sorted, 50) as u64),
      p50: Duration::from_nanos(percentile(&sorted, 50) as u64),
      p95: Duration::from_nanos(percentile(&sorted, 95) as u64),
      p99: Duration::from_nanos(percentile(&sorted, 99) as u64),
      stddev: samples.clone().std_dev(),
      ops_per_sec: if total_time.as_secs_f64() > 0.0 {
        count as f64 / total_time.as_secs_f64()
      } else {
        0.0
      },
      samples,
    })
  }

  pub fn bench_memory() -> Result<BenchResult> {
    let mem_kb = Self::read_vm_rss()?;
    let mem_bytes = mem_kb * 1024;

    Ok(BenchResult {
      category: BenchCategory::Memory,
      label: "memory_usage".into(),
      iterations: 1,
      warmup_iterations: 0,
      total_time: Duration::ZERO,
      avg_time: Duration::ZERO,
      min_time: Duration::ZERO,
      max_time: Duration::ZERO,
      median_time: Duration::ZERO,
      p50: Duration::ZERO,
      p95: Duration::ZERO,
      p99: Duration::ZERO,
      stddev: 0.0,
      ops_per_sec: mem_bytes as f64,
      samples: vec![],
    })
  }

  pub fn bench_startup(iterations: u64) -> Result<BenchResult> {
    let start = Instant::now();
    let mut samples = Vec::new();

    for _ in 0..iterations {
      let op_start = Instant::now();
      let mut child = std::process::Command::new("true")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to spawn process")?;
      child.wait().context("Failed to wait for process")?;
      samples.push(op_start.elapsed().as_secs_f64() * 1_000_000_000.0);
    }

    let total_time = start.elapsed();
    let sorted = {
      let mut s = samples.clone();
      s.sort_by(|a, b| a.partial_cmp(b).unwrap());
      s
    };

    Ok(BenchResult {
      category: BenchCategory::Startup,
      label: "process_startup".into(),
      iterations,
      warmup_iterations: 0,
      total_time,
      avg_time: total_time / iterations as u32,
      min_time: Duration::from_nanos(sorted.first().copied().unwrap_or(0.0) as u64),
      max_time: Duration::from_nanos(sorted.last().copied().unwrap_or(0.0) as u64),
      median_time: Duration::from_nanos(percentile(&sorted, 50) as u64),
      p50: Duration::from_nanos(percentile(&sorted, 50) as u64),
      p95: Duration::from_nanos(percentile(&sorted, 95) as u64),
      p99: Duration::from_nanos(percentile(&sorted, 99) as u64),
      stddev: samples.clone().std_dev(),
      ops_per_sec: if total_time.as_nanos() > 0 {
        iterations as f64 / total_time.as_secs_f64()
      } else {
        0.0
      },
      samples,
    })
  }

  pub fn compare_with_baseline(&self, dir: &Path) -> Result<BenchReport> {
    let current = BenchmarkRunner::run_all(dir)?;

    let baseline = if let Some(ref baseline_path) = self.config.baseline_path {
      if baseline_path.exists() {
        let content = std::fs::read_to_string(baseline_path)?;
        Some(serde_json::from_str::<BenchReport>(&content)?)
      } else {
        None
      }
    } else {
      None
    };

    let comparisons = if let Some(ref base) = baseline {
      let mut comps = Vec::new();
      for cur in &current.results {
        if let Some(b) = base.results.iter().find(|b| b.label == cur.label) {
          let base_avg = b.avg_time.as_secs_f64();
          let cur_avg = cur.avg_time.as_secs_f64();
          let change_pct = if base_avg > 0.0 {
            ((cur_avg - base_avg) / base_avg) * 100.0
          } else {
            0.0
          };
          comps.push(BenchComparison {
            label: cur.label.clone(),
            baseline_avg: b.avg_time,
            current_avg: cur.avg_time,
            change_pct,
            regression: change_pct > 5.0,
          });
        }
      }
      comps
    } else {
      Vec::new()
    };

    let mut report = current;
    report.baseline = baseline.map(Box::new);
    report.comparisons = comparisons;

    if let Some(ref json_path) = self.config.json_output {
      let json = serde_json::to_string_pretty(&report)?;
      std::fs::write(json_path, json).context("failed to write JSON output")?;
    }

    Ok(report)
  }

  fn parse_url(url: &str) -> Result<(String, u16, String)> {
    let url = url.trim();
    let without_scheme = url
      .strip_prefix("http://")
      .or_else(|| url.strip_prefix("https://"))
      .unwrap_or(url);
    let (host_port, path) = match without_scheme.split_once('/') {
      Some((hp, rest)) => (hp, format!("/{rest}")),
      None => (without_scheme, "/".to_string()),
    };
    let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
      let port: u16 = p.parse().context("Invalid port")?;
      (h.to_string(), port)
    } else {
      (host_port.to_string(), 80u16)
    };
    Ok((host, port, path))
  }

  fn read_vm_rss() -> Result<u64> {
    let file = std::fs::File::open("/proc/self/status")
      .context("Cannot open /proc/self/status (Linux only)")?;
    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(file);
    for line in reader.lines() {
      let line = line?;
      if let Some(value) = line.strip_prefix("VmRSS:") {
        let value = value.trim().trim_end_matches(" kB");
        return value.parse::<u64>().context("Failed to parse VmRSS");
      }
    }
    anyhow::bail!("VmRSS not found in /proc/self/status");
  }
}

impl Default for BenchmarkRunner {
  fn default() -> Self {
    Self::new()
  }
}

fn percentile(sorted: &[f64], p: usize) -> f64 {
  if sorted.is_empty() {
    return 0.0;
  }
  let rank = (p as f64 / 100.0) * (sorted.len() - 1) as f64;
  let k = rank.floor() as usize;
  let frac = rank - k as f64;
  if k + 1 < sorted.len() {
    sorted[k] + frac * (sorted[k + 1] - sorted[k])
  } else {
    sorted[k]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_run_micro() {
    let mut counter = 0u64;
    let result = BenchmarkRunner::run_micro("test_inc", &mut || {
      counter += 1;
    }).expect("Micro benchmark failed");
    assert!(result.iterations > 0);
    assert!(result.total_time > Duration::ZERO);
    assert_eq!(result.category, BenchCategory::Micro);
    assert_eq!(result.label, "test_inc");
    assert!(result.warmup_iterations > 0);
    assert!(result.avg_time.as_nanos() > 0);
  }

  #[test]
  fn test_bench_memory() {
    let result = BenchmarkRunner::bench_memory();
    if result.is_ok() {
      let r = result.unwrap();
      assert_eq!(r.category, BenchCategory::Memory);
      assert!(r.ops_per_sec > 0.0);
    }
  }

  #[test]
  fn test_bench_startup() {
    let result = BenchmarkRunner::bench_startup(2).expect("Startup benchmark failed");
    assert_eq!(result.category, BenchCategory::Startup);
    assert!(result.iterations >= 2);
    assert!(result.total_time > Duration::ZERO);
    assert!(result.min_time <= result.max_time);
  }

  #[test]
  fn test_percentile() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    assert!((percentile(&data, 50) - 5.0).abs() < 1.0);
    assert!((percentile(&data, 95) - 10.0).abs() < 1.0);
    assert!((percentile(&data, 99) - 10.0).abs() < 1.0);
  }

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
    let deserialized: BenchReport = serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(deserialized.results.len(), 1);
    assert_eq!(deserialized.results[0].label, "test");
    assert!(deserialized.results[0].samples.len() == 3);
  }
}
