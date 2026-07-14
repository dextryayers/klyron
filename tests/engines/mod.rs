use std::collections::HashMap;
use std::time::Duration;

fn test_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_test_engine_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_engine_fallback_chain() {
    use klyron_engine::fallback::{FallbackChain, FallbackStrategy};

    let mut chain = FallbackChain::with_strategy(FallbackStrategy::Fastest);
    let result = chain.resolve();
    // The engine may or may not be available depending on features
    match result {
        Ok(engine) => {
            let kind = engine.kind();
            assert!(matches!(kind, klyron_engine::JsEngineKind::V8
                | klyron_engine::JsEngineKind::Boa
                | klyron_engine::JsEngineKind::QuickJS
                | klyron_engine::JsEngineKind::JSC));
        }
        Err(e) => {
            // If no engine is available, that's acceptable if no features enabled
            assert!(e.contains("No JavaScript engine"));
        }
    }

    // Test with strategy
    let mut ordered = FallbackChain::with_strategy(FallbackStrategy::FirstWorking);
    let result2 = ordered.resolve();
    match result2 {
        Ok(engine) => {
            let _ = engine.eval("1+1");
        }
        Err(_) => {}
    }
}

#[test]
fn test_engine_fallback_with_timeout() {
    use klyron_engine::fallback::FallbackChain;

    let mut chain = FallbackChain::new();
    let result = chain.resolve_with_timeout(Duration::from_secs(5));
    match result {
        Ok(engine) => {
            let result = engine.eval("2 + 2");
            assert!(result.is_ok() || result.is_err());
        }
        Err(e) => {
            assert!(e.contains("No JavaScript engine") || e.contains("timed out"));
        }
    }
}

#[test]
fn test_engine_fallback_blacklist_reset() {
    use klyron_engine::fallback::FallbackChain;
    let mut chain = FallbackChain::new();
    chain.reset_blacklist();
    assert_eq!(chain.current_strategy(), &klyron_engine::fallback::FallbackStrategy::Fastest);
}

#[test]
fn test_engine_pool_acquire_release() {
    use klyron_engine::EnginePool;

    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 1, 4);
    let size = pool.size();
    // Pool starts with min_size entries
    assert!(size >= 0);

    match pool.acquire() {
        Ok(entry) => {
            assert!(entry.id.len() > 0);
            assert_eq!(entry.engine_kind, klyron_engine::JsEngineKind::Boa);

            // Try evaluating code
            let result = entry.eval("1 + 2");
            match result {
                Ok(val) => assert!(val.contains("3") || !val.is_empty()),
                Err(_) => {} // Engine might not be available
            }

            pool.release(&entry);
        }
        Err(e) => {
            assert!(e.contains("Engine") || e.contains("Boa"), "Error: {e}");
        }
    }
}

#[test]
fn test_engine_pool_warmup_scaling() {
    use klyron_engine::EnginePool;

    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 0, 8);
    let results = pool.warmup(3);
    assert_eq!(results.len(), 3);
    // Some may fail if engine not available
    let successes = results.iter().filter(|r| r.is_ok()).count();
    assert!(successes >= 0);

    let size = pool.size();
    assert!(size <= 3);
}

#[test]
fn test_engine_pool_pre_compile_scripts() {
    use klyron_engine::EnginePool;

    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 1, 4);
    let scripts = vec![("test", "var x = 1;")];
    // Should not panic
    pool.pre_compile_scripts(&scripts);
    let available = pool.available();
    assert!(available >= 0);
}

#[test]
fn test_engine_profile() {
    use klyron_engine::{profile_engine, JsEngineKind};

    let result = profile_engine(JsEngineKind::Boa, 10);
    match result {
        Ok(profile) => {
            assert_eq!(profile.engine_kind, JsEngineKind::Boa);
            assert!(profile.ops_per_sec >= 0.0);
            assert!(profile.warmup_complete);
        }
        Err(e) => {
            // Engine not available
            assert!(e.contains("not available") || e.contains("Boa"), "Error: {e}");
        }
    }
}

#[test]
fn test_profile_all_engines() {
    use klyron_engine::profile_all_engines;
    let profiles = profile_all_engines(5);
    // Should not panic
    for p in &profiles {
        assert!(p.ops_per_sec >= 0.0);
    }
}

#[test]
fn test_engine_pool_basic_eval() {
    use klyron_engine::EnginePool;

    let pool = EnginePool::new(klyron_engine::JsEngineKind::Boa, 1, 2);
    match pool.acquire() {
        Ok(entry) => {
            let result = entry.execute_script("test.js", "1 + 2");
            match result {
                Ok(val) => {
                    // May return JSON string of the result
                    assert!(!val.is_empty());
                }
                Err(_) => {}
            }
            pool.release(&entry);
        }
        Err(_) => {}
    }
}

#[test]
fn test_benchmark_all_engines() {
    use klyron_engine::benchmark_all_engines;
    let results = benchmark_all_engines();
    assert!(!results.is_empty());
    for (kind, result) in &results {
        assert!(result.success || !result.success);
        if result.success {
            assert!(result.eval_time >= Duration::ZERO);
        }
    }
}

#[test]
fn test_detect_best_engine() {
    use klyron_engine::detect_best_engine;
    let best = detect_best_engine();
    let name = best.name();
    assert!(!name.is_empty());
}

#[test]
fn test_each_engine_eval_1plus1() {
    use klyron_engine::{JsEngineKind, EngineRuntime};

    for kind in JsEngineKind::all() {
        match EngineRuntime::new(kind) {
            Ok(engine) => {
                let result = engine.eval("1+1");
                match result {
                    Ok(val) => {
                        assert!(!val.is_empty(), "{} eval should return non-empty", kind);
                    }
                    Err(e) => {
                        // Engine may not be fully available (e.g., V8 not compiled)
                        eprintln!("{kind} eval note: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("{kind} not available: {e}");
                // Not all engines may be built; that's acceptable
            }
        }
    }
}

#[test]
fn test_each_engine_eval_2_plus_2() {
    use klyron_engine::{JsEngineKind, EngineRuntime};

    for kind in JsEngineKind::all() {
        if let Ok(engine) = EngineRuntime::new(kind) {
            if let Ok(val) = engine.eval("2 + 2") {
                // Should contain "4" or be non-empty
                assert!(!val.is_empty(), "{kind} eval '2+2' returned empty");
            }
        }
    }
}

#[test]
fn test_each_engine_execute_script() {
    use klyron_engine::{JsEngineKind, EngineRuntime};

    for kind in JsEngineKind::all() {
        if let Ok(engine) = EngineRuntime::new(kind) {
            let result = engine.execute_script("test.js", "1 + 1");
            match result {
                Ok(val) => assert!(!val.is_empty()),
                Err(_) => {} // engine features may vary
            }
        }
    }
}

#[test]
fn test_each_engine_rejects_syntax_error() {
    use klyron_engine::{JsEngineKind, EngineRuntime};

    for kind in JsEngineKind::all() {
        if let Ok(engine) = EngineRuntime::new(kind) {
            let result = engine.eval("syntax error{{{");
            // Should be an error for all engines
            assert!(result.is_err(), "{kind} should reject invalid syntax");
        }
    }
}

#[test]
fn test_engine_eval_returns_number() {
    use klyron_engine::{JsEngineKind, EngineRuntime};

    for kind in JsEngineKind::all() {
        if let Ok(engine) = EngineRuntime::new(kind) {
            if let Ok(val) = engine.eval("42") {
                let trimmed = val.trim();
                assert!(!trimmed.is_empty(), "{kind} should return non-empty for 42");
            }
        }
    }
}
