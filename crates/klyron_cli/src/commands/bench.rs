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
            bench_http()?;
            bench_memory()?;
            bench_startup()?;
            Ok(())
        }
        Some("runtime") => bench_runtime(),
        Some("http") => bench_http(),
        Some("memory") => bench_memory(),
        Some("startup") => bench_startup(),
        Some(cat) => anyhow::bail!(
            "Unknown benchmark category: {cat}. Use: runtime, http, memory, startup"
        ),
    }
}

fn bench_runtime() -> anyhow::Result<()> {
    println!("Runtime Benchmark");
    let result = BenchmarkRunner::run_runtime("runtime_bench", &mut || {
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
    println!("  (requires running server)");
    println!("  Run `klyron serve --port 3000` in another terminal, then:");
    println!("  $ wrk -t4 -c100 -d10s http://localhost:3000/");
    Ok(())
}

fn bench_memory() -> anyhow::Result<()> {
    println!("Memory Benchmark");
    match BenchmarkRunner::bench_memory() {
        Ok(result) => println!("  Memory usage: {:.1} MB", result.ops_per_sec / 1_048_576.0),
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
