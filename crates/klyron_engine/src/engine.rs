use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
struct V8EngineWrapper(klyron_engine_v8::V8Engine);

#[cfg(feature = "v8")]
impl JsEngineInternal for V8EngineWrapper {
    fn eval_inner(&mut self, code: &str) -> Result<String, String> {
        self.0.eval(code).map_err(|e| e.to_string())
    }
    fn execute_script_inner(&mut self, filename: &str, source: &str) -> Result<String, String> {
        self.0.execute_script(filename, source).map_err(|e| e.to_string())
    }
}

#[cfg(feature = "v8")]
type V8Adapter = Adapter<V8EngineWrapper>;

#[cfg(feature = "v8")]
impl V8Adapter {
    fn new() -> Result<Self, String> {
        klyron_engine_v8::V8Engine::new().map(V8EngineWrapper).map(Adapter::wrap).map_err(|e| e.to_string())
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
