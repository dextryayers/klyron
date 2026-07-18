use crate::report::BenchResult;
use crate::BenchCategory;
use crate::BenchConfig;
use anyhow::{Context, Result};
use std::io::Read;
use std::time::{Duration, Instant};
use statrs::statistics::Statistics;

pub struct BenchmarkRunner {
    config: BenchConfig,
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        BenchmarkRunner { config: BenchConfig::default() }
    }

    pub fn with_config(config: BenchConfig) -> Self {
        BenchmarkRunner { config }
    }

    pub fn config(&self) -> &BenchConfig {
        &self.config
    }

    pub fn run<F>(&self, label: &str, category: BenchCategory, mut f: F) -> Result<BenchResult>
    where
        F: FnMut(),
    {
        for _ in 0..self.config.warmup_iterations {
            (f)();
        }

        let mut samples = Vec::new();
        let mut min_time = Duration::MAX;
        let mut max_time = Duration::ZERO;
        let mut total_time = Duration::ZERO;
        let mut iterations: u64 = 0;
        let batch_size = self.config.batch_size;

        while total_time < self.config.min_duration
            || iterations < self.config.min_iterations
        {
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

        let samples_ns: Vec<f64> = samples;
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
            category,
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

    pub fn run_micro(label: &str, f: &mut dyn FnMut()) -> Result<BenchResult> {
        let runner = BenchmarkRunner::with_config(BenchConfig::default());
        runner.run(label, BenchCategory::Micro, f)
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
                let request = format!(
                    "GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n"
                );
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

pub fn percentile(sorted: &[f64], p: usize) -> f64 {
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
        })
        .expect("Micro benchmark failed");
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
        let result =
            BenchmarkRunner::bench_startup(2).expect("Startup benchmark failed");
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
    }
}
