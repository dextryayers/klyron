/// Script characteristics extracted from source analysis
#[derive(Debug, Clone)]
pub struct ScriptFeatures {
    pub byte_size: usize,
    pub has_loops: bool,
    pub has_promises: bool,
    pub has_async: bool,
    pub import_count: usize,
    pub top_level_await: bool,
    pub expected_runtime_ms: f64,
}

impl ScriptFeatures {
    pub fn new() -> Self {
        Self {
            byte_size: 0,
            has_loops: false,
            has_promises: false,
            has_async: false,
            import_count: 0,
            top_level_await: false,
            expected_runtime_ms: 0.0,
        }
    }
}

/// Classifies scripts by statically analyzing source code patterns
pub struct ScriptClassifier;

impl ScriptClassifier {
    pub fn new() -> Self {
        Self
    }

    /// Extract features from JavaScript/TypeScript source code
    pub fn extract(&self, code: &str) -> ScriptFeatures {
        let mut features = ScriptFeatures::new();
        features.byte_size = code.len();

        // Detect loops
        features.has_loops = code.contains("for(")
            || code.contains("for (")
            || code.contains("while(")
            || code.contains("while (")
            || code.contains("do{")
            || code.contains("do {");

        // Detect promises
        features.has_promises = code.contains(".then(")
            || code.contains(".catch(")
            || code.contains(".finally(")
            || code.contains("new Promise(")
            || code.contains("Promise.resolve(")
            || code.contains("Promise.all(")
            || code.contains("Promise.race(");

        // Detect async/await
        features.has_async = code.contains("async ")
            || code.contains("await ")
            || code.contains("async(");

        // Count imports
        features.import_count = code.lines()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("import ") || t.starts_with("const ") || t.starts_with("let ")
            })
            .count();

        // Detect top-level await
        features.top_level_await = code.contains("await ") && !code.contains("async ");

        // Estimate runtime based on script size and complexity
        features.expected_runtime_ms = Self::estimate_runtime(&features);

        features
    }

    /// Estimate expected runtime in milliseconds based on features
    fn estimate_runtime(features: &ScriptFeatures) -> f64 {
        let base = features.byte_size as f64 * 0.01; // ~0.01ms per byte
        let loop_penalty = if features.has_loops { 50.0 } else { 0.0 };
        let promise_penalty = if features.has_promises { 20.0 } else { 0.0 };
        let async_penalty = if features.has_async { 10.0 } else { 0.0 };
        let import_penalty = features.import_count as f64 * 5.0;

        (base + loop_penalty + promise_penalty + async_penalty + import_penalty).max(1.0)
    }
}

/// Exponential Moving Average for latency tracking
#[derive(Debug, Clone)]
pub struct ExponentialMovingAverage {
    value: f64,
    alpha: f64,  // smoothing factor (0.0 - 1.0)
    initialized: bool,
}

impl ExponentialMovingAverage {
    pub fn new(alpha: f64) -> Self {
        Self { value: 0.0, alpha, initialized: false }
    }

    pub fn update(&mut self, sample: f64) -> f64 {
        if !self.initialized {
            self.value = sample;
            self.initialized = true;
        } else {
            self.value = self.alpha * sample + (1.0 - self.alpha) * self.value;
        }
        self.value
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
        self.initialized = false;
    }
}

use crate::JsEngineKind;

/// Predict the best engine for given script features
pub fn predict_engine(features: &ScriptFeatures) -> JsEngineKind {
    match features {
        // Very short scripts (< 5ms expected): QuickJS (fastest startup)
        f if f.expected_runtime_ms < 5.0 => JsEngineKind::QuickJS,
        // Loops + long runtime: V8 (JIT compilation wins)
        f if f.has_loops && f.expected_runtime_ms > 100.0 => JsEngineKind::V8,
        // Async with moderate runtime: Boa (readable stack traces for debugging)
        f if f.has_async && f.expected_runtime_ms < 50.0 => JsEngineKind::Boa,
        // Batch (many imports, no interactivity): JSC
        f if f.import_count > 50 => JsEngineKind::JSC,
        // Default: V8
        _ => JsEngineKind::V8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_simple() {
        let classifier = ScriptClassifier::new();
        let features = classifier.extract("1 + 1");
        assert!(!features.has_loops);
        assert!(!features.has_async);
        assert_eq!(features.import_count, 0);
    }

    #[test]
    fn test_classify_with_loops() {
        let classifier = ScriptClassifier::new();
        let features = classifier.extract("for (let i = 0; i < 10; i++) { sum += i; }");
        assert!(features.has_loops);
    }

    #[test]
    fn test_classify_with_async() {
        let classifier = ScriptClassifier::new();
        let features = classifier.extract("async function fetchData() { const r = await fetch('/api'); }");
        assert!(features.has_async);
    }

    #[test]
    fn test_predict_short_script() {
        let mut features = ScriptFeatures::new();
        features.expected_runtime_ms = 1.0;
        assert_eq!(predict_engine(&features), JsEngineKind::QuickJS);
    }

    #[test]
    fn test_predict_v8_for_loops() {
        let mut features = ScriptFeatures::new();
        features.has_loops = true;
        features.expected_runtime_ms = 200.0;
        assert_eq!(predict_engine(&features), JsEngineKind::V8);
    }

    #[test]
    fn test_ema() {
        let mut ema = ExponentialMovingAverage::new(0.3);
        ema.update(100.0);
        assert!((ema.value() - 100.0).abs() < 0.01);
        ema.update(50.0);
        assert!((ema.value() - 85.0).abs() < 0.01);
    }
}
