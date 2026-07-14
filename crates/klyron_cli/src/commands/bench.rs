use clap::Args;

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
        Some(cat) => anyhow::bail!("Unknown benchmark category: {cat}. Use: runtime, http, memory, startup"),
    }
}

fn bench_runtime() -> anyhow::Result<()> {
    println!("📊 Runtime Benchmark");
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let runtime = klyron_core::Runtime::builder().build()?;
        runtime.eval("1+1")?;
    }
    let elapsed = start.elapsed();
    println!("  1000 iterations: {:?} ({:?} avg)", elapsed, elapsed / 1000);
    Ok(())
}

fn bench_http() -> anyhow::Result<()> {
    println!("📊 HTTP Benchmark");
    println!("  (requires running server)");
    println!("  Run `klyron serve --port 3000` in another terminal, then:");
    println!("  $ wrk -t4 -c100 -d10s http://localhost:3000/");
    Ok(())
}

fn bench_memory() -> anyhow::Result<()> {
    println!("📊 Memory Benchmark");
    let runtime = klyron_core::Runtime::builder().build()?;
    runtime.eval("const arr = new Array(1000000).fill('hello'); arr.length")?;
    println!("  Array of 1M strings allocated");
    drop(runtime);
    Ok(())
}

fn bench_startup() -> anyhow::Result<()> {
    println!("📊 Startup Benchmark");
    let start = std::time::Instant::now();
    let runtime = klyron_core::Runtime::builder().build()?;
    runtime.eval("1+1")?;
    let elapsed = start.elapsed();
    println!("  Cold start: {:?}", elapsed);
    let start = std::time::Instant::now();
    runtime.eval("1+1")?;
    let elapsed = start.elapsed();
    println!("  Warm eval: {:?}", elapsed);
    Ok(())
}
