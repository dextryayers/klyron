use std::time::{Duration, Instant};
use klyron_engine::{JsEngineKind, EngineRuntime, detect_best_engine};

fn bench_engine_initialization(kind: JsEngineKind, iterations: u64) {
    let mut total = Duration::ZERO;
    for i in 0..iterations {
        let start = Instant::now();
        let engine = EngineRuntime::new(kind).expect("Engine init failed");
        let elapsed = start.elapsed();
        total += elapsed;
        if i == 0 {
            let _ = engine.eval("1+1");
        }
    }
    let avg = total / iterations as u32;
    println!("  {kind:<12} init: {avg:?} avg ({iterations} iterations)");
}

fn bench_engine_eval_simple(kind: JsEngineKind, code: &str, iterations: u64) {
    let engine = EngineRuntime::new(kind).expect("Engine init failed");
    for _ in 0..100 { let _ = engine.eval(code); }
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = engine.eval(code);
    }
    let elapsed = start.elapsed();
    let avg = elapsed / iterations as u32;
    let ops = (iterations as f64 / elapsed.as_secs_f64()) as u64;
    println!("  {kind:<12} eval simple: {avg:?} avg, {ops} ops/s ({iterations} iterations)");
}

fn bench_engine_eval_complex(kind: JsEngineKind, code: &str, iterations: u64) {
    let engine = EngineRuntime::new(kind).expect("Engine init failed");
    for _ in 0..20 { let _ = engine.eval(code); }
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = engine.eval(code);
    }
    let elapsed = start.elapsed();
    let avg = elapsed / iterations as u32;
    println!("  {kind:<12} eval complex: {avg:?} avg ({iterations} iterations)");
}

fn bench_engine_module_loading(kind: JsEngineKind) {
    let engine = EngineRuntime::new(kind).expect("Engine init failed");
    let module_code = "export const foo = 42;";
    let import_code = "import { foo } from './module.js'; console.log(foo);";
    let start = Instant::now();
    let iterations = 1000u64;
    for _ in 0..iterations {
        let _ = engine.execute_script("module.js", module_code);
        let _ = engine.execute_script("main.js", import_code);
    }
    let elapsed = start.elapsed();
    let avg = elapsed / iterations as u32;
    println!("  {kind:<12} module loading: {avg:?} avg ({iterations} iterations)");
}

fn bench_bytecode_cache() {
    let code = "function fib(n) { return n < 2 ? n : fib(n-1) + fib(n-2); } fib(30);";
    let kind = detect_best_engine();
    let engine = EngineRuntime::new(kind).expect("Engine init failed");

    let start = Instant::now();
    let _ = engine.eval(code);
    let no_cache = start.elapsed();

    let start = Instant::now();
    let _ = engine.eval(code);
    let cached = start.elapsed();

    let speedup = if cached > Duration::ZERO && no_cache > cached {
        (no_cache.as_nanos() as f64 / cached.as_nanos() as f64)
    } else {
        1.0
    };

    println!("  Bytecode cache: no_cache={no_cache:?}, cached={cached:?}, speedup={speedup:.2}x");
}

fn bench_parallel_resolution() {
    let kind = detect_best_engine();
    use rayon::prelude::*;
    let codes: Vec<String> = (0..100).map(|i| format!("var x{i} = {i}; x{i};")).collect();

    let start = Instant::now();
    let _results: Vec<_> = codes.par_iter().map(|code| {
        let engine = EngineRuntime::new(kind).expect("Engine init failed");
        engine.eval(code).ok()
    }).collect();
    let parallel = start.elapsed();

    let start = Instant::now();
    for code in &codes {
        let engine = EngineRuntime::new(kind).expect("Engine init failed");
        let _ = engine.eval(code);
    }
    let sequential = start.elapsed();

    println!("  Parallel resolution: {parallel:?} (sequential: {sequential:?})");
}

fn main() {
    let engines = JsEngineKind::all();
    println!("\n=== Engine Initialization ===");
    for kind in &engines {
        bench_engine_initialization(*kind, 50);
    }

    println!("\n=== Engine Eval Simple (1+1) ===");
    for kind in &engines {
        bench_engine_eval_simple(*kind, "1+1;", 5000);
    }

    println!("\n=== Engine Eval Complex (fib(30)) ===");
    let fib_code = "function fib(n) { return n < 2 ? n : fib(n-1) + fib(n-2); } fib(30);";
    for kind in &engines {
        bench_engine_eval_complex(*kind, fib_code, 200);
    }

    println!("\n=== Engine Module Loading ===");
    for kind in &engines {
        bench_engine_module_loading(*kind);
    }

    println!("\n=== Bytecode Cache Comparison ===");
    bench_bytecode_cache();

    println!("\n=== Parallel Resolution ===");
    bench_parallel_resolution();
}
