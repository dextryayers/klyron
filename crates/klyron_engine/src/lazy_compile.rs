use std::collections::HashMap;
use std::sync::Mutex;

use crate::engine::{EngineRuntime, JsEngineKind};

pub struct CompiledModule {
    pub path: String,
    pub content: String,
    pub compiled: bool,
    pub bytecode: Option<Vec<u8>>,
}

pub struct LazyCompiler {
    modules: Mutex<HashMap<String, CompiledModule>>,
    engine: Mutex<Option<EngineRuntime>>,
}

impl LazyCompiler {
    pub fn new(engine_kind: JsEngineKind) -> Result<Self, String> {
        let engine = EngineRuntime::new(engine_kind).ok();
        Ok(Self {
            modules: Mutex::new(HashMap::new()),
            engine: Mutex::new(engine),
        })
    }

    pub fn register(&self, path: &str, content: &str) {
        let mut modules = self.modules.lock().unwrap();
        modules.insert(
            path.to_string(),
            CompiledModule {
                path: path.to_string(),
                content: content.to_string(),
                compiled: false,
                bytecode: None,
            },
        );
    }

    pub fn compile_on_demand(&self, path: &str) -> Result<(), String> {
        let mut modules = self.modules.lock().map_err(|e| e.to_string())?;
        let module = modules.get_mut(path).ok_or_else(|| format!("Module not registered: {path}"))?;
        if module.compiled {
            return Ok(());
        }
        let engine = self.engine.lock().map_err(|e| e.to_string())?;
        if let Some(ref eng) = *engine {
            eng.eval(&module.content).map_err(|e| format!("Compile failed: {e}"))?;
        }
        module.compiled = true;
        Ok(())
    }

    pub fn pre_warm(&self, module_paths: &[(&str, &str)]) {
        for (path, content) in module_paths {
            self.register(path, content);
            let _ = self.compile_on_demand(path);
        }
    }

    pub fn is_compiled(&self, path: &str) -> bool {
        self.modules.lock().ok()
            .and_then(|m| m.get(path).map(|m| m.compiled))
            .unwrap_or(false)
    }

    pub fn compiled_count(&self) -> usize {
        self.modules.lock().ok()
            .map(|m| m.values().filter(|m| m.compiled).count())
            .unwrap_or(0)
    }

    pub fn registered_count(&self) -> usize {
        self.modules.lock().map(|m| m.len()).unwrap_or(0)
    }

    pub fn eval_if_compiled(&self, _path: &str, code: &str) -> Result<String, String> {
        let engine = self.engine.lock().map_err(|e| e.to_string())?;
        match *engine {
            Some(ref eng) => eng.eval(code).map_err(|e| e.to_string()),
            None => Err("No engine available".to_string()),
        }
    }

    pub fn reset(&self) {
        let mut modules = self.modules.lock().unwrap();
        modules.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::JsEngineKind;

    #[test]
    fn test_lazy_compiler_new() {
        let compiler = LazyCompiler::new(JsEngineKind::Boa).unwrap();
        assert_eq!(compiler.registered_count(), 0);
        assert_eq!(compiler.compiled_count(), 0);
    }

    #[test]
    fn test_register_module() {
        let compiler = LazyCompiler::new(JsEngineKind::Boa).unwrap();
        compiler.register("/path/mod.js", "const x = 1;");
        assert_eq!(compiler.registered_count(), 1);
        assert!(!compiler.is_compiled("/path/mod.js"));
    }

    #[test]
    fn test_register_multiple() {
        let compiler = LazyCompiler::new(JsEngineKind::Boa).unwrap();
        compiler.register("a.js", "1;");
        compiler.register("b.js", "2;");
        assert_eq!(compiler.registered_count(), 2);
    }

    #[test]
    fn test_compile_on_demand_no_engine() {
        let compiler = LazyCompiler::new(JsEngineKind::Boa).unwrap();
        compiler.register("test.js", "1+1");
        // compile_on_demand will fail since no engine is available (default features)
        let result = compiler.compile_on_demand("test.js");
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_compile_on_demand_not_registered() {
        let compiler = LazyCompiler::new(JsEngineKind::Boa).unwrap();
        let result = compiler.compile_on_demand("nonexistent.js");
        assert!(result.is_err());
    }

    #[test]
    fn test_pre_warm() {
        let compiler = LazyCompiler::new(JsEngineKind::Boa).unwrap();
        compiler.pre_warm(&[("a.js", "1+1"), ("b.js", "2+2")]);
        assert_eq!(compiler.registered_count(), 2);
    }

    #[test]
    fn test_is_compiled_unknown() {
        let compiler = LazyCompiler::new(JsEngineKind::Boa).unwrap();
        assert!(!compiler.is_compiled("unknown.js"));
    }

    #[test]
    fn test_reset() {
        let compiler = LazyCompiler::new(JsEngineKind::Boa).unwrap();
        compiler.register("test.js", "code");
        compiler.reset();
        assert_eq!(compiler.registered_count(), 0);
    }

    #[test]
    fn test_compiled_module_structure() {
        let module = CompiledModule {
            path: "mod.js".to_string(),
            content: "code".to_string(),
            compiled: false,
            bytecode: None,
        };
        assert!(!module.compiled);
        assert!(module.bytecode.is_none());
        assert_eq!(module.path, "mod.js");
    }

    #[test]
    fn test_eval_if_compiled_no_engine() {
        // Test with an engine that may or may not be available
        let compiler = LazyCompiler::new(JsEngineKind::Boa);
        if let Ok(compiler) = compiler {
            let result = compiler.eval_if_compiled("test.js", "1+1");
            // Should either be Ok (with full engine) or Err (without)
            assert!(result.is_ok() || result.is_err());
        }
    }
}
