use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_v8_eval(c: &mut Criterion) {
    let engine = klyron_engine::EngineRuntime::new(klyron_engine::JsEngineKind::V8).unwrap();
    c.bench_function("v8_eval_simple", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("1 + 2 + 3"));
        })
    });
    c.bench_function("v8_eval_function", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("(function(a,b){return a*b;})(6,7)"));
        })
    });
    c.bench_function("v8_eval_object", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("JSON.stringify({a:1,b:2,c:3})"));
        })
    });
    c.bench_function("v8_eval_array_ops", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("[1,2,3,4,5].map(x=>x*x).reduce((a,b)=>a+b,0)"));
        })
    });
    c.bench_function("v8_eval_fib", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("function fib(n){return n<2?n:fib(n-1)+fib(n-2)};fib(20)"));
        })
    });
    c.bench_function("v8_eval_loop", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("let s=0;for(let i=0;i<1000;i++)s+=i;s"));
        })
    });
}

fn bench_boa_eval(c: &mut Criterion) {
    let engine = klyron_engine::EngineRuntime::new(klyron_engine::JsEngineKind::Boa).unwrap();
    let mut group = c.benchmark_group("boa");
    group.bench_function("eval_simple", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("1 + 2 + 3"));
        })
    });
    group.bench_function("eval_function", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("(function(a,b){return a*b;})(6,7)"));
        })
    });
    group.bench_function("eval_object", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("JSON.stringify({a:1,b:2,c:3})"));
        })
    });
    group.bench_function("eval_array_ops", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("[1,2,3,4,5].map(x=>x*x).reduce((a,b)=>a+b,0)"));
        })
    });
    group.bench_function("eval_fib", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("function fib(n){return n<2?n:fib(n-1)+fib(n-2)};fib(20)"));
        })
    });
    group.bench_function("eval_loop", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("let s=0;for(let i=0;i<1000;i++)s+=i;s"));
        })
    });
    group.finish();
}

fn bench_quickjs_eval(c: &mut Criterion) {
    let engine = klyron_engine::EngineRuntime::new(klyron_engine::JsEngineKind::QuickJS).unwrap();
    let mut group = c.benchmark_group("quickjs");
    group.bench_function("eval_simple", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("1 + 2 + 3"));
        })
    });
    group.bench_function("eval_function", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("(function(a,b){return a*b;})(6,7)"));
        })
    });
    group.bench_function("eval_object", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("JSON.stringify({a:1,b:2,c:3})"));
        })
    });
    group.bench_function("eval_array_ops", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("[1,2,3,4,5].map(x=>x*x).reduce((a,b)=>a+b,0)"));
        })
    });
    group.bench_function("eval_fib", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("function fib(n){return n<2?n:fib(n-1)+fib(n-2)};fib(20)"));
        })
    });
    group.bench_function("eval_loop", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("let s=0;for(let i=0;i<1000;i++)s+=i;s"));
        })
    });
    group.finish();
}

fn bench_jsc_eval(c: &mut Criterion) {
    let engine = klyron_engine::EngineRuntime::new(klyron_engine::JsEngineKind::JSC).unwrap();
    let mut group = c.benchmark_group("jsc");
    group.bench_function("eval_simple", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("1 + 2 + 3"));
        })
    });
    group.bench_function("eval_function", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("(function(a,b){return a*b;})(6,7)"));
        })
    });
    group.bench_function("eval_object", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("JSON.stringify({a:1,b:2,c:3})"));
        })
    });
    group.bench_function("eval_array_ops", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("[1,2,3,4,5].map(x=>x*x).reduce((a,b)=>a+b,0)"));
        })
    });
    group.bench_function("eval_fib", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("function fib(n){return n<2?n:fib(n-1)+fib(n-2)};fib(20)"));
        })
    });
    group.bench_function("eval_loop", |b| {
        b.iter(|| {
            let _ = engine.eval(black_box("let s=0;for(let i=0;i<1000;i++)s+=i;s"));
        })
    });
    group.finish();
}

fn bench_startup_time(c: &mut Criterion) {
    c.bench_function("startup_v8", |b| {
        b.iter(|| {
            let _ = klyron_engine::EngineRuntime::new(klyron_engine::JsEngineKind::V8);
        })
    });
    c.bench_function("startup_boa", |b| {
        b.iter(|| {
            let _ = klyron_engine::EngineRuntime::new(klyron_engine::JsEngineKind::Boa);
        })
    });
    c.bench_function("startup_quickjs", |b| {
        b.iter(|| {
            let _ = klyron_engine::EngineRuntime::new(klyron_engine::JsEngineKind::QuickJS);
        })
    });
    c.bench_function("startup_jsc", |b| {
        b.iter(|| {
            let _ = klyron_engine::EngineRuntime::new(klyron_engine::JsEngineKind::JSC);
        })
    });
}

fn bench_detect_best(c: &mut Criterion) {
    c.bench_function("detect_best_engine", |b| {
        b.iter(|| {
            let _ = klyron_engine::detect_best_engine();
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(100).warm_up_time(std::time::Duration::from_secs(2)).measurement_time(std::time::Duration::from_secs(5));
    targets = bench_startup_time, bench_detect_best,
);
criterion_group!(
    name = v8;
    config = Criterion::default().sample_size(100);
    targets = bench_v8_eval,
);
criterion_group!(
    name = boa;
    config = Criterion::default().sample_size(50).warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_boa_eval,
);
criterion_group!(
    name = quickjs;
    config = Criterion::default().sample_size(50).warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_quickjs_eval,
);
criterion_group!(
    name = jsc;
    config = Criterion::default().sample_size(50).warm_up_time(std::time::Duration::from_secs(1));
    targets = bench_jsc_eval,
);

criterion_main!(benches, v8, boa, quickjs, jsc);
