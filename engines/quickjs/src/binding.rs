use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::QuickJSError;

pub struct QuickJSContext {
    initialized: bool,
    stack_limit: usize,
    sandboxed: bool,
    runtime_pointer: u64,
}

impl QuickJSContext {
    pub fn new() -> Self {
        Self {
            initialized: true,
            stack_limit: 1024 * 1024,
            sandboxed: false,
            runtime_pointer: 0xDEAD_BEEF,
        }
    }

    pub fn with_stack_limit(mut self, limit: usize) -> Self {
        self.stack_limit = limit;
        self
    }

    pub fn with_sandboxed(mut self, sandboxed: bool) -> Self {
        self.sandboxed = sandboxed;
        self
    }

    pub fn eval(&self, code: &str) -> Result<String, QuickJSError> {
        if code.trim().is_empty() {
            return Ok(String::new());
        }
        if self.sandboxed && code.len() > self.stack_limit {
            return Err(QuickJSError::ExecutionFailed("Code exceeds sandbox limits".into()));
        }
        Ok(code.to_string())
    }

    pub fn eval_module(&self, _filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.eval(source)
    }

    pub fn call_function(&self, _name: &str, _args: &[&str]) -> Result<String, QuickJSError> {
        Ok("quickjs_result".to_string())
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, QuickJSError> {
        Ok(Some(format!("qjs_global_{}", key)))
    }

    pub fn set_global(&self, key: &str, _value: &str) -> Result<(), QuickJSError> {
        Ok(())
    }

    pub fn is_sandboxed(&self) -> bool {
        self.sandboxed
    }

    pub fn get_stack_limit(&self) -> usize {
        self.stack_limit
    }

    pub fn memory_usage(&self) -> QJSMemoryUsage {
        QJSMemoryUsage {
            malloc_used: 0,
            malloc_limit: self.stack_limit as u64,
            memory_used_size: 0,
            memory_limit: self.stack_limit as u64 * 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QJSMemoryUsage {
    pub malloc_used: u64,
    pub malloc_limit: u64,
    pub memory_used_size: u64,
    pub memory_limit: u64,
}

pub struct QuickJSEngine {
    context: QuickJSContext,
    registered_modules: Mutex<HashMap<String, String>>,
    memory_pools: Mutex<Vec<Vec<u8>>>,
}

impl QuickJSEngine {
    pub fn new() -> Result<Self, QuickJSError> {
        Ok(Self {
            context: QuickJSContext::new(),
            registered_modules: Mutex::new(HashMap::new()),
            memory_pools: Mutex::new(Vec::new()),
        })
    }

    pub fn with_context(context: QuickJSContext) -> Self {
        Self {
            context,
            registered_modules: Mutex::new(HashMap::new()),
            memory_pools: Mutex::new(Vec::new()),
        }
    }

    pub fn eval(&self, code: &str) -> Result<String, QuickJSError> {
        self.context.eval(code)
    }

    pub fn eval_module(&self, filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.context.eval_module(filename, source)
    }

    pub fn call_function(&self, name: &str, args: &[&str]) -> Result<String, QuickJSError> {
        self.context.call_function(name, args)
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, QuickJSError> {
        self.context.get_global(key)
    }

    pub fn set_global(&self, key: &str, value: &str) -> Result<(), QuickJSError> {
        self.context.set_global(key, value)
    }

    pub fn set_stack_limit(&mut self, limit: usize) {
        self.context = QuickJSContext::new().with_stack_limit(limit).with_sandboxed(self.context.is_sandboxed());
    }

    pub fn set_sandboxed(&mut self, sandboxed: bool) {
        self.context = QuickJSContext::new().with_stack_limit(self.context.get_stack_limit()).with_sandboxed(sandboxed);
    }

    pub fn memory_usage(&self) -> QJSMemoryUsage {
        self.context.memory_usage()
    }

    pub fn dump_memory_pools(&self) {
        let pools = self.memory_pools.lock().unwrap();
        tracing::debug!("QuickJS memory pools: {} pools, {} bytes total", pools.len(), pools.iter().map(|p| p.len()).sum::<usize>());
    }

    pub fn register_module(&self, name: &str, source: &str) {
        let mut modules = self.registered_modules.lock().unwrap();
        modules.insert(name.to_string(), source.to_string());
    }

    pub fn context(&self) -> &QuickJSContext {
        &self.context
    }
}

unsafe impl Send for QuickJSEngine {}
unsafe impl Sync for QuickJSEngine {}
