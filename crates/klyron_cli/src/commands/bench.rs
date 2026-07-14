use clap::Args;
use klyron_bench::BenchmarkRunner;

#[derive(Args)]
pub struct BenchArgs {
    pub category: Option<String>,
}

pub fn run_bench(args: BenchArgs) -> anyhow::Result<()> {
    match args.category.as_deref() {
        None | Some("all") => {
            bench_runtime()?;
            bench_engine()?;
            bench_http()?;
            bench_memory()?;
            bench_startup()?;
            Ok(())
        }
        Some("runtime") => bench_runtime(),
        Some("engine") => bench_engine(),
        Some("http") => bench_http(),
        Some("memory") => bench_memory(),
        Some("startup") => bench_startup(),
        Some(cat) => anyhow::bail!(
            "Unknown benchmark category: {cat}. Use: runtime, engine, http, memory, startup"
        ),
    }
}

fn bench_engine() -> anyhow::Result<()> {
    println!("JS Engine Benchmark (via klyron_engine)");
    let results = klyron_engine::benchmark_all_engines();
    let mut sorted: Vec<_> = results.into_iter().collect();
    sorted.sort_by_key(|(_, r)| r.eval_time);
    for (kind, result) in &sorted {
        let status = if result.success { "OK" } else { "FAIL" };
        println!("  {kind:<8} {status}  {:>12?}", result.eval_time);
    }
    if let Some((best, _)) = sorted.first() {
        println!("  Best engine: {best}");
    }
    Ok(())
}

fn bench_runtime() -> anyhow::Result<()> {
    println!("Runtime Benchmark");
    let result = BenchmarkRunner::run_micro("runtime_bench", &mut || {
        let _ = 1 + 1;
    })?;
    println!(
        "  {} iterations: {:?} ({:?} avg, {:.0} ops/sec)",
        result.iterations, result.total_time, result.avg_time, result.ops_per_sec
    );
    Ok(())
}

fn bench_http() -> anyhow::Result<()> {
    println!("HTTP Benchmark");
    let url = "http://localhost:3000/";
    println!("  Benchmarking {url} (10s, 10 connections)");
    match BenchmarkRunner::bench_http(url, 10, std::time::Duration::from_secs(10)) {
        Ok(result) => {
            println!("  Requests: {}", result.iterations);
            println!("  Total time: {:.2}s", result.total_time.as_secs_f64());
            println!("  Avg latency: {:?}", result.avg_time);
            println!("  Min latency: {:?}", result.min_time);
            println!("  Max latency: {:?}", result.max_time);
            println!("  Requests/sec: {:.0}", result.ops_per_sec);
        }
        Err(e) => println!("  Could not benchmark HTTP: {e}"),
    }
    Ok(())
}

fn bench_memory() -> anyhow::Result<()> {
    println!("Memory Benchmark");
    match BenchmarkRunner::bench_memory() {
        Ok(result) => println!("  Memory usage: {:.1} MB", result.ops_per_sec / (1024.0 * 1024.0)),
        Err(e) => println!("  Could not measure memory: {e}"),
    }
    Ok(())
}

fn bench_startup() -> anyhow::Result<()> {
    println!("Startup Benchmark");
    let result = BenchmarkRunner::bench_startup(10)?;
    println!(
        "  Cold start ({:?} avg across {} iterations)",
        result.avg_time, result.iterations
    );
    Ok(())
}
