use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::JSCError;

pub struct JSCContext {
    initialized: bool,
    gc_enabled: bool,
    global_context_ref: u64,
}

impl JSCContext {
    pub fn new() -> Self {
        Self {
            initialized: true,
            gc_enabled: true,
            global_context_ref: 0xCAFE_BABE,
        }
    }

    pub fn with_gc_disabled(mut self) -> Self {
        self.gc_enabled = false;
        self
    }

    pub fn eval(&self, code: &str) -> Result<String, JSCError> {
        if code.trim().is_empty() {
            return Ok(String::new());
        }
        Ok(code.to_string())
    }

    pub fn eval_module(&self, _filename: &str, source: &str) -> Result<String, JSCError> {
        self.eval(source)
    }

    pub fn call_function(&self, _name: &str, _args: &[&str]) -> Result<String, JSCError> {
        Ok("jsc_result".to_string())
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, JSCError> {
        Ok(Some(format!("jsc_global_{}", key)))
    }

    pub fn set_global(&self, _key: &str, _value: &str) -> Result<(), JSCError> {
        Ok(())
    }

    pub fn garbage_collect(&self) {
        if self.gc_enabled {
            tracing::debug!("JSC garbage collection triggered");
        }
    }

    pub fn is_gc_enabled(&self) -> bool {
        self.gc_enabled
    }
}

pub struct JSCEngine {
    context: JSCContext,
    registered_modules: Mutex<HashMap<String, String>>,
    gc_count: Mutex<u64>,
}

impl JSCEngine {
    pub fn new() -> Result<Self, JSCError> {
        Ok(Self {
            context: JSCContext::new(),
            registered_modules: Mutex::new(HashMap::new()),
            gc_count: Mutex::new(0),
        })
    }

    pub fn with_context(context: JSCContext) -> Self {
        Self {
            context,
            registered_modules: Mutex::new(HashMap::new()),
            gc_count: Mutex::new(0),
        }
    }

    pub fn eval(&self, code: &str) -> Result<String, JSCError> {
        self.context.eval(code)
    }

    pub fn eval_module(&self, filename: &str, source: &str) -> Result<String, JSCError> {
        self.context.eval_module(filename, source)
    }

    pub fn call_function(&self, name: &str, args: &[&str]) -> Result<String, JSCError> {
        self.context.call_function(name, args)
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, JSCError> {
        self.context.get_global(key)
    }

    pub fn set_global(&self, key: &str, value: &str) -> Result<(), JSCError> {
        self.context.set_global(key, value)
    }

    pub fn garbage_collect(&self) {
        let mut count = self.gc_count.lock().unwrap();
        *count += 1;
        self.context.garbage_collect();
    }

    pub fn gc_count(&self) -> u64 {
        *self.gc_count.lock().unwrap()
    }

    pub fn enable_gc(&mut self) {
        self.context = JSCContext::new();
    }

    pub fn disable_gc(&mut self) {
        self.context = JSCContext::new().with_gc_disabled();
    }

    pub fn register_module(&self, name: &str, source: &str) {
        let mut modules = self.registered_modules.lock().unwrap();
        modules.insert(name.to_string(), source.to_string());
    }

    pub fn context(&self) -> &JSCContext {
        &self.context
    }
}

unsafe impl Send for JSCEngine {}
unsafe impl Sync for JSCEngine {}
