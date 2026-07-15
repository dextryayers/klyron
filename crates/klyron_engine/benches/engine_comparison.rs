use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_instantiation(c: &mut Criterion) {
    for kind in klyron_engine::JsEngineKind::all() {
        let name = format!("instantiation_{}", kind.name());
        c.bench_function(&name, |b| {
            b.iter(|| {
                let _ = klyron_engine::EngineRuntime::new(kind);
            })
        });
    }
}

fn bench_eval_simple(c: &mut Criterion) {
    for kind in klyron_engine::JsEngineKind::all() {
        if let Ok(engine) = klyron_engine::EngineRuntime::new(kind) {
            let name = format!("{}/eval(1+1)", kind.name());
            c.bench_function(&name, |b| {
                b.iter(|| {
                    let _ = engine.eval(black_box("1+1"));
                })
            });
        }
    }
}

fn bench_module_load(c: &mut Criterion) {
    let module_code = r#"
        export const PI = 3.14159;
        export function add(a, b) { return a + b; }
        export class Point {
            constructor(x, y) { this.x = x; this.y = y; }
            dist() { return Math.sqrt(this.x*this.x + this.y*this.y); }
        }
    "#;
    for kind in klyron_engine::JsEngineKind::all() {
        if let Ok(engine) = klyron_engine::EngineRuntime::new(kind) {
            let name = format!("{}/module_load", kind.name());
            c.bench_function(&name, |b| {
                b.iter(|| {
                    let _ = engine.execute_script(black_box("module.js"), black_box(module_code));
                })
            });
        }
    }
}

fn bench_gc_pressure(c: &mut Criterion) {
    let gc_code = r#"
        let arr = [];
        for (let i = 0; i < 10000; i++) {
            arr.push({a: i, b: {c: i * 2, d: String(i)}});
        }
        arr = null;
    "#;
    for kind in klyron_engine::JsEngineKind::all() {
        if let Ok(engine) = klyron_engine::EngineRuntime::new(kind) {
            let name = format!("{}/gc_pressure", kind.name());
            c.bench_function(&name, |b| {
                b.iter(|| {
                    let _ = engine.eval(black_box(gc_code));
                })
            });
        }
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(100).warm_up_time(std::time::Duration::from_secs(2)).measurement_time(std::time::Duration::from_secs(5));
    targets = bench_instantiation, bench_eval_simple, bench_module_load, bench_gc_pressure,
);

criterion_main!(benches);
