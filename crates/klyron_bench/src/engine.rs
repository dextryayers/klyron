use crate::report::BenchResult;
use crate::runner::percentile;
use crate::BenchCategory;
use anyhow::Result;
use klyron_engine::engine::{JsEngineKind, EngineRuntime};
use statrs::statistics::Statistics;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct EngineBenchGroup {
    pub engine: JsEngineKind,
    pub startup: Option<BenchResult>,
    pub eval: Option<BenchResult>,
    pub memory: Option<BenchResult>,
}

impl BenchmarkRunnerExt {
    pub fn bench_engine_startup(kind: JsEngineKind, iterations: u64) -> Result<BenchResult> {
        let start = Instant::now();
        let mut samples = Vec::new();

        for _ in 0..iterations {
            let op_start = Instant::now();
            let _engine = EngineRuntime::new(kind)
                .map_err(|e| anyhow::anyhow!("Engine {} init failed: {}", kind, e))?;
            let elapsed = op_start.elapsed();
            samples.push(elapsed.as_secs_f64() * 1_000_000_000.0);
        }

        let total_time = start.elapsed();
        let sorted = {
            let mut s = samples.clone();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s
        };

        let mean = samples.clone().mean();
        let stddev = samples.clone().std_dev();

        Ok(BenchResult {
            category: BenchCategory::Startup,
            label: format!("engine_startup_{}", kind),
            iterations,
            warmup_iterations: 0,
            total_time,
            avg_time: Duration::from_nanos(mean as u64),
            min_time: Duration::from_nanos(sorted.first().copied().unwrap_or(0.0) as u64),
            max_time: Duration::from_nanos(sorted.last().copied().unwrap_or(0.0) as u64),
            median_time: Duration::from_nanos(percentile(&sorted, 50) as u64),
            p50: Duration::from_nanos(percentile(&sorted, 50) as u64),
            p95: Duration::from_nanos(percentile(&sorted, 95) as u64),
            p99: Duration::from_nanos(percentile(&sorted, 99) as u64),
            stddev,
            ops_per_sec: if mean > 0.0 { 1_000_000_000.0 / mean } else { 0.0 },
            samples,
        })
    }

    pub fn bench_engine_eval(
        kind: JsEngineKind,
        code: &str,
        iterations: u64,
    ) -> Result<BenchResult> {
        let engine = EngineRuntime::new(kind)
            .map_err(|e| anyhow::anyhow!("Engine {} init failed: {}", kind, e))?;

        for _ in 0..100u64.min(iterations) {
            let _ = engine.eval(code);
        }

        let start = Instant::now();
        let mut samples = Vec::new();
        let batch_size = 100u64;
        let mut total: u64 = 0;

        while start.elapsed() < Duration::from_secs(2) || total < iterations {
            let batch_start = Instant::now();
            for _ in 0..batch_size {
                let _ = engine.eval(code);
            }
            let elapsed = batch_start.elapsed();
            let per_op = elapsed / batch_size as u32;
            samples.push(per_op.as_secs_f64() * 1_000_000_000.0);
            total += batch_size;
        }

        let total_time = start.elapsed();
        let sorted = {
            let mut s = samples.clone();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s
        };
        let mean = samples.clone().mean();
        let stddev = samples.clone().std_dev();

        Ok(BenchResult {
            category: BenchCategory::Runtime,
            label: format!("engine_eval_{}", kind),
            iterations: total,
            warmup_iterations: 0,
            total_time,
            avg_time: Duration::from_nanos(mean as u64),
            min_time: Duration::from_nanos(sorted.first().copied().unwrap_or(0.0) as u64),
            max_time: Duration::from_nanos(sorted.last().copied().unwrap_or(0.0) as u64),
            median_time: Duration::from_nanos(percentile(&sorted, 50) as u64),
            p50: Duration::from_nanos(percentile(&sorted, 50) as u64),
            p95: Duration::from_nanos(percentile(&sorted, 95) as u64),
            p99: Duration::from_nanos(percentile(&sorted, 99) as u64),
            stddev,
            ops_per_sec: if mean > 0.0 { 1_000_000_000.0 / mean } else { 0.0 },
            samples,
        })
    }

    pub fn bench_engine_memory(kind: JsEngineKind) -> Result<BenchResult> {
        let mem_before = read_vm_rss()?;
        let _engine = EngineRuntime::new(kind)
            .map_err(|e| anyhow::anyhow!("Engine {} init failed: {}", kind, e))?;
        let mem_after = read_vm_rss()?;
        let mem_used = mem_after.saturating_sub(mem_before) * 1024;

        Ok(BenchResult {
            category: BenchCategory::Memory,
            label: format!("engine_memory_{}", kind),
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
            ops_per_sec: mem_used as f64,
            samples: vec![],
        })
    }

    pub fn bench_all_engines() -> Result<HashMap<JsEngineKind, EngineBenchGroup>> {
        let mut results = HashMap::new();
        let bench_code = "
            function fib(n) { return n < 2 ? n : fib(n-1) + fib(n-2); }
            fib(20);
        ";

        for kind in JsEngineKind::all() {
            let startup = Self::bench_engine_startup(kind, 10).ok();
            let eval = Self::bench_engine_eval(kind, bench_code, 500).ok();
            let memory = Self::bench_engine_memory(kind).ok();

            results.insert(
                kind,
                EngineBenchGroup {
                    engine: kind,
                    startup,
                    eval,
                    memory,
                },
            );
        }

        Ok(results)
    }
}

pub struct BenchmarkRunnerExt;

fn read_vm_rss() -> anyhow::Result<u64> {
    let file = std::fs::File::open("/proc/self/status")
        .map_err(|e| anyhow::anyhow!("Cannot open /proc/self/status: {}", e))?;
    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        if let Some(value) = line.strip_prefix("VmRSS:") {
            let value = value.trim().trim_end_matches(" kB");
            return value.parse::<u64>().map_err(|e| anyhow::anyhow!("Parse error: {}", e));
        }
    }
    anyhow::bail!("VmRSS not found in /proc/self/status");
}
