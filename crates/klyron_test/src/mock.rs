use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static MOCK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct MockCall {
    pub args: Vec<serde_json::Value>,
    pub timestamp: u128,
}

#[derive(Debug, Clone)]
pub struct MockFunction {
    pub id: u64,
    pub name: String,
    pub calls: Vec<MockCall>,
    pub mock_implementation: Option<String>,
    pub return_value: Option<serde_json::Value>,
}

impl MockFunction {
    pub fn new(name: &str) -> Self {
        Self {
            id: MOCK_ID.fetch_add(1, Ordering::SeqCst),
            name: name.to_string(),
            calls: Vec::new(),
            mock_implementation: None,
            return_value: None,
        }
    }

    pub fn with_return(mut self, value: serde_json::Value) -> Self {
        self.return_value = Some(value);
        self
    }

    pub fn call(&mut self, args: Vec<serde_json::Value>) -> Option<serde_json::Value> {
        self.calls.push(MockCall {
            args,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        });
        self.return_value.clone()
    }

    pub fn called(&self) -> bool {
        !self.calls.is_empty()
    }

    pub fn called_once(&self) -> bool {
        self.calls.len() == 1
    }

    pub fn called_times(&self, n: usize) -> bool {
        self.calls.len() == n
    }

    pub fn called_with(&self, args: &[serde_json::Value]) -> bool {
        self.calls.iter().any(|call| call.args == args)
    }

    pub fn reset(&mut self) {
        self.calls.clear();
    }
}

pub struct MockRegistry {
    mocks: RefCell<HashMap<String, MockFunction>>,
}

impl MockRegistry {
    pub fn new() -> Self {
        Self {
            mocks: RefCell::new(HashMap::new()),
        }
    }

    pub fn register(&self, name: &str, mock: MockFunction) {
        self.mocks.borrow_mut().insert(name.to_string(), mock);
    }

    pub fn get(&self, name: &str) -> Option<MockFunction> {
        self.mocks.borrow().get(name).cloned()
    }

    pub fn get_mut(&self, _name: &str) -> Option<std::cell::RefMut<'_, MockFunction>> {
        None
    }

    pub fn reset_all(&self) {
        self.mocks.borrow_mut().clear();
    }

    pub fn all_mocks(&self) -> Vec<MockFunction> {
        self.mocks.borrow().values().cloned().collect()
    }
}

impl Default for MockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate_mock_globals() -> String {
    r#"
(function() {
    const __klyron_mocks = {};

    globalThis.vi = {
        fn(impl) {
            const mock = (...args) => {
                mock.calls.push(args);
                if (mock.impl) mock.impl(...args);
                return mock.returnValue;
            };
            mock.calls = [];
            mock.impl = impl || null;
            mock.returnValue = undefined;
            mock.mockReturnValue = function(val) {
                mock.returnValue = val;
                return mock;
            };
            mock.mockImplementation = function(fn) {
                mock.impl = fn;
                return mock;
            };
            return mock;
        },
        spyOn(obj, method) {
            const original = obj[method];
            const mock = globalThis.vi.fn(original);
            obj[method] = mock;
            return mock;
        },
        clearAllMocks() {
            Object.values(__klyron_mocks).forEach(m => m.calls = []);
        },
        resetAllMocks() {
            Object.values(__klyron_mocks).forEach(m => {
                m.calls = [];
                m.impl = null;
                m.returnValue = undefined;
            });
        },
    };
})();
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_function_new() {
        let mock = MockFunction::new("testFn");
        assert_eq!(mock.name, "testFn");
        assert!(!mock.called());
    }

    #[test]
    fn test_mock_function_call() {
        let mut mock = MockFunction::new("fn").with_return(serde_json::json!(42));
        let result = mock.call(vec![serde_json::json!(1)]);
        assert_eq!(result, Some(serde_json::json!(42)));
        assert!(mock.called());
        assert!(mock.called_once());
        assert!(mock.called_with(&[serde_json::json!(1)]));
    }

    #[test]
    fn test_mock_registry() {
        let registry = MockRegistry::new();
        let mock = MockFunction::new("myMock");
        registry.register("myMock", mock);
        assert!(registry.get("myMock").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_generate_mock_globals() {
        let globals = generate_mock_globals();
        assert!(globals.contains("globalThis.vi"));
        assert!(globals.contains("mockReturnValue"));
        assert!(globals.contains("mockImplementation"));
    }
}
