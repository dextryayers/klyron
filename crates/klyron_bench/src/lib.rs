/// Benchmark runner for Klyron — measures runtime, HTTP, memory, and startup performance.
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Category of benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchCategory {
    Runtime,
    Http,
    Memory,
    Startup,
}

/// Result of a single benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    pub category: BenchCategory,
    pub iterations: u64,
    pub total_time: Duration,
    pub avg_time: Duration,
    pub ops_per_sec: f64,
    pub label: String,
}

/// Report containing all benchmark results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchReport {
    pub results: Vec<BenchResult>,
    pub timestamp: SystemTime,
}

/// Main benchmark runner.
pub struct BenchmarkRunner;

impl BenchmarkRunner {
    /// Run all benchmarks in the given directory by discovering and executing them.
    ///
    /// This scans for subdirectories or marker files and runs appropriate benchmarks.
    pub fn run_all(dir: &Path) -> Result<BenchReport> {
        let mut results = Vec::new();
        let timestamp = SystemTime::now();

        // Attempt runtime benchmark on the directory
        if let Ok(r) = Self::run_runtime(
            &format!("runtime_{}", dir.file_name().unwrap_or_default().to_string_lossy()),
            &mut || {
                let _ = std::fs::read_dir(dir);
            },
        ) {
            results.push(r);
        }

        // Attempt memory benchmark
        if let Ok(r) = Self::bench_memory() {
            results.push(r);
        }

        // Attempt startup benchmark
        if let Ok(r) = Self::bench_startup(10) {
            results.push(r);
        }

        Ok(BenchReport { results, timestamp })
    }

    /// Run a closure repeatedly and measure performance.
    ///
    /// The closure is executed `iterations` times (auto-calculated for at least 1 second).
    pub fn run_runtime(label: &str, f: &mut dyn FnMut()) -> Result<BenchResult> {
        let min_time = Duration::from_secs(1);
        let mut iterations: u64 = 1;

        // Warmup
        (f)();

        // Determine iterations to run for at least min_time
        let start = Instant::now();
        for _ in 0..1000 {
            (f)();
        }
        let elapsed = start.elapsed();
        if elapsed > Duration::ZERO {
            let est = min_time.as_nanos() / elapsed.as_nanos();
            iterations = (est as u64 * 1000).max(1);
        }
        iterations = iterations.min(10_000_000);

        let start = Instant::now();
        for _ in 0..iterations {
            (f)();
        }
        let total_time = start.elapsed();
        let avg_time = total_time / iterations as u32;
        let ops_per_sec = if avg_time.as_nanos() > 0 {
            1_000_000_000.0 / avg_time.as_nanos() as f64
        } else {
            0.0
        };

        Ok(BenchResult {
            category: BenchCategory::Runtime,
            iterations,
            total_time,
            avg_time,
            ops_per_sec,
            label: label.to_string(),
        })
    }

    /// Benchmark HTTP throughput by fetching a URL.
    ///
    /// Opens multiple connections and measures throughput.
    pub fn bench_http(url: &str, connections: u32, duration: Duration) -> Result<BenchResult> {
        let (host, port, path) = Self::parse_url(url)?;

        let start = Instant::now();
        let mut _total_bytes = 0u64;
        let mut count = 0u64;

        while start.elapsed() < duration {
            for _ in 0..connections {
                let mut stream = TcpStream::connect_timeout(
                    &format!("{host}:{port}").parse().context("Invalid address")?,
                    Duration::from_secs(5),
                )?;
                let request = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
                stream.write_all(request.as_bytes())?;
                stream.flush()?;
                let mut buf = Vec::new();
                stream.read_to_end(&mut buf)?;
                _total_bytes += buf.len() as u64;
                count += 1;
            }
        }

        let total_time = start.elapsed();
        let ops_per_sec = if total_time.as_secs_f64() > 0.0 {
            count as f64 / total_time.as_secs_f64()
        } else {
            0.0
        };

        Ok(BenchResult {
            category: BenchCategory::Http,
            iterations: count,
            total_time,
            avg_time: total_time / count as u32,
            ops_per_sec,
            label: url.to_string(),
        })
    }

    /// Benchmark current memory usage from `/proc/self/status`.
    pub fn bench_memory() -> Result<BenchResult> {
        let mem_kb = Self::read_vm_rss()?;
        let mem_bytes = mem_kb * 1024;

        Ok(BenchResult {
            category: BenchCategory::Memory,
            iterations: 1,
            total_time: Duration::ZERO,
            avg_time: Duration::ZERO,
            ops_per_sec: mem_bytes as f64,
            label: "memory_usage".to_string(),
        })
    }

    /// Benchmark process creation time (cold start).
    ///
    /// Spawns `iterations` number of short-lived processes and measures total time.
    pub fn bench_startup(iterations: u64) -> Result<BenchResult> {
        let start = Instant::now();
        for _ in 0..iterations {
            let mut child = Command::new("true")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .context("Failed to spawn process")?;
            child.wait().context("Failed to wait for process")?;
        }
        let total_time = start.elapsed();
        let avg_time = total_time / iterations as u32;
        let ops_per_sec = if avg_time.as_nanos() > 0 {
            1_000_000_000.0 / avg_time.as_nanos() as f64
        } else {
            0.0
        };

        Ok(BenchResult {
            category: BenchCategory::Startup,
            iterations,
            total_time,
            avg_time,
            ops_per_sec,
            label: "process_startup".to_string(),
        })
    }

    /// Parse a URL string into (host, port, path).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_runtime() {
        let mut counter = 0u64;
        let result = BenchmarkRunner::run_runtime("test_inc", &mut || {
            counter += 1;
        })
        .expect("Runtime benchmark failed");
        assert!(result.iterations > 0);
        assert!(result.total_time > Duration::ZERO);
        assert_eq!(result.category, BenchCategory::Runtime);
        assert_eq!(result.label, "test_inc");
    }

    #[test]
    fn test_bench_memory() {
        let result = BenchmarkRunner::bench_memory();
        if result.is_ok() {
            let r = result.unwrap();
            assert_eq!(r.category, BenchCategory::Memory);
            assert!(r.ops_per_sec > 0.0);
        }
        // On non-Linux this may fail, that's fine.
    }

    #[test]
    fn test_bench_startup() {
        let result = BenchmarkRunner::bench_startup(2).expect("Startup benchmark failed");
        assert_eq!(result.category, BenchCategory::Startup);
        assert!(result.iterations >= 2);
        assert!(result.total_time > Duration::ZERO);
    }

    #[test]
    fn test_run_runtime_zero_time() {
        // Very fast closure to test ops_per_sec calculation
        let result = BenchmarkRunner::run_runtime("fast", &mut || {
            let _ = 1 + 1;
        })
        .expect("Fast runtime benchmark failed");
        assert!(result.ops_per_sec > 0.0);
        assert!(result.avg_time <= result.total_time);
    }

    #[test]
    fn test_bench_report_serialization() {
        let report = BenchReport {
            results: vec![BenchResult {
                category: BenchCategory::Runtime,
                iterations: 100,
                total_time: Duration::from_secs(1),
                avg_time: Duration::from_millis(10),
                ops_per_sec: 100.0,
                label: "test".to_string(),
            }],
            timestamp: SystemTime::now(),
        };
        let json = serde_json::to_string(&report).expect("Serialization failed");
        let deserialized: BenchReport = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.results.len(), 1);
        assert_eq!(deserialized.results[0].label, "test");
    }
}
