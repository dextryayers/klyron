use crate::traits::{JsEngine, JsError, JsValue};
use std::collections::HashMap;
use std::sync::Mutex;

/// A mock JavaScript engine for testing purposes.
/// Behaves predictably without any native dependencies.
pub struct MockEngine {
    counter: Mutex<u64>,
    responses: Mutex<HashMap<String, String>>,
}

impl MockEngine {
    pub fn new() -> Self {
        Self {
            counter: Mutex::new(0),
            responses: Mutex::new(HashMap::new()),
        }
    }

    /// Register a canned response for a specific code input.
    pub fn mock_response(&self, code: &str, response: &str) {
        let mut map = self.responses.lock().unwrap();
        map.insert(code.to_string(), response.to_string());
    }

    /// Clear all registered responses.
    pub fn clear_responses(&self) {
        let mut map = self.responses.lock().unwrap();
        map.clear();
    }

    /// Get the number of eval calls made.
    pub fn eval_count(&self) -> u64 {
        *self.counter.lock().unwrap()
    }
}

impl JsEngine for MockEngine {
    fn eval(&self, code: &str) -> Result<JsValue, JsError> {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;

        let map = self.responses.lock().unwrap();
        if let Some(response) = map.get(code) {
            return Ok(JsValue::String(response.clone()));
        }

        // Default responses for common patterns
        if code.trim() == "1 + 1" {
            return Ok(JsValue::from(2.0_f64));
        }
        if code.trim() == "typeof globalThis" {
            return Ok(JsValue::String("object".to_string()));
        }
        if code.contains("throw") || code.contains("error") {
            return Err("Mock error: code contains throw/error".to_string());
        }

        Ok(JsValue::Null)
    }

    fn execute_script(&self, _filename: &str, source: &str) -> Result<JsValue, JsError> {
        self.eval(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_engine_new() {
        let engine = MockEngine::new();
        assert_eq!(engine.eval_count(), 0);
    }

    #[test]
    fn test_mock_engine_eval_math() {
        let engine = MockEngine::new();
        let result = engine.eval("1 + 1").unwrap();
        assert_eq!(result, JsValue::from(2.0_f64));
        assert_eq!(engine.eval_count(), 1);
    }

    #[test]
    fn test_mock_engine_custom_response() {
        let engine = MockEngine::new();
        engine.mock_response("hello()", "world");
        let result = engine.eval("hello()").unwrap();
        assert_eq!(result, JsValue::String("world".to_string()));
    }

    #[test]
    fn test_mock_engine_error() {
        let engine = MockEngine::new();
        let result = engine.eval("throw new Error('test')");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_engine_execute_script() {
        let engine = MockEngine::new();
        let result = engine.execute_script("test.js", "1 + 1").unwrap();
        assert_eq!(result, JsValue::from(2.0_f64));
    }

    #[test]
    fn test_mock_engine_clear() {
        let engine = MockEngine::new();
        engine.mock_response("foo()", "bar");
        engine.clear_responses();
        let result = engine.eval("foo()");
        assert_eq!(result, Ok(JsValue::Null));
    }

    #[test]
    fn test_mock_engine_multiple_calls() {
        let engine = MockEngine::new();
        engine.eval("1 + 1").unwrap();
        engine.eval("1 + 1").unwrap();
        engine.eval("1 + 1").unwrap();
        assert_eq!(engine.eval_count(), 3);
    }
}
