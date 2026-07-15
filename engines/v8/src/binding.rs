use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::V8Error;

pub struct V8Context {
    initialized: bool,
    heap_limit: usize,
    heap_used: u64,
    snapshot_blob: Option<Vec<u8>>,
}

impl V8Context {
    pub fn new() -> Self {
        Self {
            initialized: true,
            heap_limit: 512 * 1024 * 1024,
            heap_used: 0,
            snapshot_blob: None,
        }
    }

    pub fn with_heap_limit(mut self, limit: usize) -> Self {
        self.heap_limit = limit;
        self
    }

    pub fn with_snapshot(mut self, blob: Vec<u8>) -> Self {
        self.snapshot_blob = Some(blob);
        self
    }

    pub fn eval(&self, code: &str) -> Result<String, V8Error> {
        if code.trim().is_empty() {
            return Ok(String::new());
        }
        self.heap_used += code.len() as u64;
        if self.heap_used > self.heap_limit as u64 {
            return Err(V8Error::OutOfMemory);
        }
        Ok(code.to_string())
    }

    pub fn eval_module(&self, _filename: &str, source: &str) -> Result<String, V8Error> {
        self.eval(source)
    }

    pub fn call_function(&self, _name: &str, _args: &[&str]) -> Result<String, V8Error> {
        Ok("function_result".to_string())
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, V8Error> {
        Ok(Some(format!("global_{}", key)))
    }

    pub fn set_global(&self, key: &str, _value: &str) -> Result<(), V8Error> {
        Ok(())
    }

    pub fn get_heap_stats(&self) -> HeapStatistics {
        HeapStatistics {
            used_heap_size: self.heap_used,
            total_heap_size: self.heap_limit as u64,
            heap_size_limit: self.heap_limit as u64,
            malloced_memory: 0,
            peak_malloced_memory: 0,
            number_of_native_contexts: 1,
            number_of_detached_contexts: 0,
        }
    }

    pub fn low_memory_notification(&self) {
    }

    pub fn idle_notification_deadline(&self, _deadline_in_seconds: f64) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct HeapStatistics {
    pub used_heap_size: u64,
    pub total_heap_size: u64,
    pub heap_size_limit: u64,
    pub malloced_memory: u64,
    pub peak_malloced_memory: u64,
    pub number_of_native_contexts: u32,
    pub number_of_detached_contexts: u32,
}

pub struct V8Engine {
    context: V8Context,
    snapshots_enabled: bool,
    registered_modules: Mutex<HashMap<String, String>>,
}

impl V8Engine {
    pub fn new() -> Result<Self, V8Error> {
        Ok(Self {
            context: V8Context::new(),
            snapshots_enabled: true,
            registered_modules: Mutex::new(HashMap::new()),
        })
    }

    pub fn with_context(context: V8Context) -> Self {
        Self {
            context,
            snapshots_enabled: true,
            registered_modules: Mutex::new(HashMap::new()),
        }
    }

    pub fn eval(&self, code: &str) -> Result<String, V8Error> {
        self.context.eval(code)
    }

    pub fn eval_module(&self, filename: &str, source: &str) -> Result<String, V8Error> {
        self.context.eval_module(filename, source)
    }

    pub fn call_function(&self, name: &str, args: &[&str]) -> Result<String, V8Error> {
        self.context.call_function(name, args)
    }

    pub fn get_global(&self, key: &str) -> Result<Option<String>, V8Error> {
        self.context.get_global(key)
    }

    pub fn set_global(&self, key: &str, value: &str) -> Result<(), V8Error> {
        self.context.set_global(key, value)
    }

    pub fn create_snapshot(&self) -> Result<Vec<u8>, V8Error> {
        Ok(bincode::serialize(&self.context.get_heap_stats()).unwrap_or_default())
    }

    pub fn load_snapshot(&mut self, blob: Vec<u8>) -> Result<(), V8Error> {
        self.context = V8Context::new().with_snapshot(blob);
        Ok(())
    }

    pub fn heap_statistics(&self) -> HeapStatistics {
        self.context.get_heap_stats()
    }

    pub fn register_module(&self, name: &str, source: &str) {
        let mut modules = self.registered_modules.lock().unwrap();
        modules.insert(name.to_string(), source.to_string());
    }

    pub fn get_heap_limit(&self) -> usize {
        self.context.heap_limit
    }

    pub fn context(&self) -> &V8Context {
        &self.context
    }
}

unsafe impl Send for V8Engine {}
unsafe impl Sync for V8Engine {}
