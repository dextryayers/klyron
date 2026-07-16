use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum JsEngineKind {
    V8,
    Boa,
    QuickJS,
    JSC,
}

impl JsEngineKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::V8 => "v8",
            Self::Boa => "boa",
            Self::QuickJS => "quickjs",
            Self::JSC => "jsc",
        }
    }

    pub fn all() -> Vec<JsEngineKind> {
        vec![Self::V8, Self::Boa, Self::QuickJS, Self::JSC]
    }
}

impl std::fmt::Display for JsEngineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

pub type JsValue = serde_json::Value;
pub type JsError = String;

pub trait JsEngine {
    fn eval(&self, code: &str) -> Result<JsValue, JsError>;
    fn execute_script(&self, filename: &str, source: &str) -> Result<JsValue, JsError>;
}

#[derive(Debug, Clone)]
pub struct BenchResult {
    pub eval_time: std::time::Duration,
    pub success: bool,
    pub error: Option<String>,
}

pub struct EngineRuntime {
    kind: JsEngineKind,
    inner: Box<dyn JsEngine>,
}

impl EngineRuntime {
    pub fn new(kind: JsEngineKind) -> Result<Self, String> {
        if kind == JsEngineKind::V8 {
            #[cfg(feature = "v8")]
            { return Ok(Self { kind, inner: Box::new(V8Adapter::new()?) }); }
        }
        if kind == JsEngineKind::Boa {
            #[cfg(feature = "boa")]
            { return Ok(Self { kind, inner: Box::new(BoaAdapter::new()?) }); }
        }
        if kind == JsEngineKind::QuickJS {
            #[cfg(feature = "quickjs")]
            { return Ok(Self { kind, inner: Box::new(QuickJSAdapter::new()?) }); }
        }
        if kind == JsEngineKind::JSC {
            #[cfg(feature = "jsc")]
            { return Ok(Self { kind, inner: Box::new(JSCAdapter::new()?) }); }
        }
        Err(format!("Engine {} not available (feature not enabled)", kind))
    }

    pub fn with_fallback() -> Result<Self, String> {
        for kind in JsEngineKind::all() {
            match Self::new(kind) {
                Ok(engine) => {
                    tracing::info!("Using engine: {}", kind);
                    return Ok(engine);
                }
                Err(e) => {
                    tracing::warn!("Engine {} failed: {}", kind, e);
                    continue;
                }
            }
        }
        Err("No JavaScript engine available".to_string())
    }

    pub fn kind(&self) -> JsEngineKind {
        self.kind
    }

    pub fn eval(&self, code: &str) -> Result<String, String> {
        let result = self.inner.eval(code)?;
        Ok(serde_json::to_string(&result).unwrap_or_else(|_| result.to_string()))
    }

    pub fn eval_json(&self, code: &str) -> Result<JsValue, String> {
        self.inner.eval(code)
    }

    pub fn execute_script(&self, filename: &str, source: &str) -> Result<String, String> {
        let result = self.inner.execute_script(filename, source)?;
        Ok(serde_json::to_string(&result).unwrap_or_else(|_| result.to_string()))
    }
}

pub fn benchmark_all_engines() -> HashMap<JsEngineKind, BenchResult> {
    let mut results = HashMap::new();
    let bench_code = "1 + 2 + 3";

    for kind in JsEngineKind::all() {
        match EngineRuntime::new(kind) {
            Ok(engine) => {
                let start = Instant::now();
                let result = engine.eval(bench_code);
                let elapsed = start.elapsed();
                let (success, error) = match result {
                    Ok(_) => (true, None),
                    Err(e) => (false, Some(e)),
                };
                results.insert(kind, BenchResult { eval_time: elapsed, success, error });
            }
            Err(e) => {
                results.insert(kind, BenchResult {
                    eval_time: std::time::Duration::default(),
                    success: false,
                    error: Some(e),
                });
            }
        }
    }

    results
}

pub fn detect_best_engine() -> JsEngineKind {
    let results = benchmark_all_engines();
    results.into_iter()
        .filter(|(_, r)| r.success)
        .min_by_key(|(_, r)| r.eval_time)
        .map(|(kind, _)| kind)
        .unwrap_or(JsEngineKind::Boa)
}

// --- Adapter implementations ---

use std::sync::Mutex;

struct Adapter<E> {
    engine: Mutex<E>,
}

impl<E> Adapter<E>
where
    E: JsEngineInternal,
{
    fn wrap(engine: E) -> Self {
        Self { engine: Mutex::new(engine) }
    }
}

trait JsEngineInternal {
    fn eval_inner(&mut self, code: &str) -> Result<String, String>;
    fn execute_script_inner(&mut self, filename: &str, source: &str) -> Result<String, String>;
}

impl<E> JsEngine for Adapter<E>
where
    E: JsEngineInternal + 'static,
{
    fn eval(&self, code: &str) -> Result<JsValue, JsError> {
        let mut engine = self.engine.lock().map_err(|e| e.to_string())?;
        let result = engine.eval_inner(code)?;
        Ok(serde_json::Value::String(result))
    }

    fn execute_script(&self, filename: &str, source: &str) -> Result<JsValue, JsError> {
        let mut engine = self.engine.lock().map_err(|e| e.to_string())?;
        let result = engine.execute_script_inner(filename, source)?;
        Ok(serde_json::Value::String(result))
    }
}

#[cfg(feature = "v8")]
struct V8EngineWrapper(std::sync::Mutex<klyron_core::Runtime>);

#[cfg(feature = "v8")]
impl JsEngineInternal for V8EngineWrapper {
    fn eval_inner(&mut self, code: &str) -> Result<String, String> {
        self.0.lock().map_err(|e| e.to_string())?.eval(code).map_err(|e| e.to_string())
    }
    fn execute_script_inner(&mut self, filename: &str, source: &str) -> Result<String, String> {
        self.0.lock().map_err(|e| e.to_string())?
            .execute_script(filename, source).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "v8")]
type V8Adapter = Adapter<V8EngineWrapper>;

#[cfg(feature = "v8")]
impl V8Adapter {
    fn new() -> Result<Self, String> {
        let runtime = klyron_core::Runtime::builder().build().map_err(|e| e.to_string())?;
        Ok(Adapter::wrap(V8EngineWrapper(std::sync::Mutex::new(runtime))))
    }
}

#[cfg(feature = "boa")]
struct BoaEngineWrapper(klyron_engine_boa::BoaEngine);

#[cfg(feature = "boa")]
impl JsEngineInternal for BoaEngineWrapper {
    fn eval_inner(&mut self, code: &str) -> Result<String, String> {
        self.0.eval(code).map_err(|e| e.to_string())
    }
    fn execute_script_inner(&mut self, filename: &str, source: &str) -> Result<String, String> {
        self.0.execute_script(filename, source).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "boa")]
type BoaAdapter = Adapter<BoaEngineWrapper>;

#[cfg(feature = "boa")]
impl BoaAdapter {
    fn new() -> Result<Self, String> {
        Ok(Adapter::wrap(BoaEngineWrapper(klyron_engine_boa::BoaEngine::new())))
    }
}

#[cfg(feature = "quickjs")]
struct QuickJSEngineWrapper(klyron_engine_quickjs::QuickJSEngine);

#[cfg(feature = "quickjs")]
impl JsEngineInternal for QuickJSEngineWrapper {
    fn eval_inner(&mut self, code: &str) -> Result<String, String> {
        self.0.eval(code).map_err(|e| e.to_string())
    }
    fn execute_script_inner(&mut self, filename: &str, source: &str) -> Result<String, String> {
        self.0.execute_script(filename, source).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "quickjs")]
type QuickJSAdapter = Adapter<QuickJSEngineWrapper>;

#[cfg(feature = "quickjs")]
impl QuickJSAdapter {
    fn new() -> Result<Self, String> {
        klyron_engine_quickjs::QuickJSEngine::new().map(QuickJSEngineWrapper).map(Adapter::wrap).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "jsc")]
struct JSCEngineWrapper(klyron_engine_jsc::JSCEngine);

#[cfg(feature = "jsc")]
impl JsEngineInternal for JSCEngineWrapper {
    fn eval_inner(&mut self, code: &str) -> Result<String, String> {
        self.0.eval(code).map_err(|e| e.to_string())
    }
    fn execute_script_inner(&mut self, filename: &str, source: &str) -> Result<String, String> {
        self.0.execute_script(filename, source).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "jsc")]
type JSCAdapter = Adapter<JSCEngineWrapper>;

#[cfg(feature = "jsc")]
impl JSCAdapter {
    fn new() -> Result<Self, String> {
        klyron_engine_jsc::JSCEngine::new().map(JSCEngineWrapper).map(Adapter::wrap).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_kind_name() {
        assert_eq!(JsEngineKind::V8.name(), "v8");
        assert_eq!(JsEngineKind::Boa.name(), "boa");
        assert_eq!(JsEngineKind::QuickJS.name(), "quickjs");
        assert_eq!(JsEngineKind::JSC.name(), "jsc");
    }

    #[test]
    fn test_engine_kind_display() {
        assert_eq!(JsEngineKind::V8.to_string(), "v8");
        assert_eq!(JsEngineKind::Boa.to_string(), "boa");
    }

    #[test]
    fn test_engine_kind_all() {
        let all = JsEngineKind::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&JsEngineKind::V8));
        assert!(all.contains(&JsEngineKind::Boa));
        assert!(all.contains(&JsEngineKind::QuickJS));
        assert!(all.contains(&JsEngineKind::JSC));
    }

    #[test]
    fn test_engine_kind_equality() {
        assert_eq!(JsEngineKind::V8, JsEngineKind::V8);
        assert_ne!(JsEngineKind::V8, JsEngineKind::Boa);
    }

    #[test]
    fn test_engine_kind_clone() {
        let kind = JsEngineKind::QuickJS;
        let cloned = kind;
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_engine_kind_serialize_roundtrip() {
        let kinds = vec![JsEngineKind::V8, JsEngineKind::Boa, JsEngineKind::QuickJS, JsEngineKind::JSC];
        for kind in kinds {
            let json = serde_json::to_string(&kind).unwrap();
            let deserialized: JsEngineKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, deserialized);
        }
    }

    #[test]
    fn test_bench_result_default() {
        let result = BenchResult {
            eval_time: std::time::Duration::from_secs(1),
            success: true,
            error: None,
        };
        assert!(result.success);
        assert!(result.error.is_none());
        assert_eq!(result.eval_time.as_secs(), 1);
    }

    #[test]
    fn test_bench_result_with_error() {
        let result = BenchResult {
            eval_time: std::time::Duration::default(),
            success: false,
            error: Some("engine failed".to_string()),
        };
        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "engine failed");
    }

    #[test]
    fn test_engine_runtime_new_unavailable() {
        let result = EngineRuntime::new(JsEngineKind::V8);
        // With default features, no engines are available
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_runtime_with_fallback_unavailable() {
        let result = EngineRuntime::with_fallback();
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_best_engine_fallback() {
        let best = detect_best_engine();
        // Without engines, falls back to Boa
        assert_eq!(best, JsEngineKind::Boa);
    }

    #[test]
    fn test_jsvalue_type_alias() {
        let val: JsValue = serde_json::json!({"key": "value"});
        assert_eq!(val["key"], "value");
    }
}
