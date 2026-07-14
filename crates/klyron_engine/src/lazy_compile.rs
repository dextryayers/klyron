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
            Some(ref eng) => eng.eval(code),
            None => Err("No engine available".to_string()),
        }
    }

    pub fn reset(&self) {
        let mut modules = self.modules.lock().unwrap();
        modules.clear();
    }
}
