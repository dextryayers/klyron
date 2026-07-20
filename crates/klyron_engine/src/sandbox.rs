use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use uuid::Uuid;

use crate::engine::{JsEngineKind, EngineRuntime, EngineError};
use crate::memory_limits::MemoryLimits;

#[derive(Debug, Clone)]
pub struct SandboxContext {
    pub id: String,
    pub engine_kind: JsEngineKind,
    pub created_at: Instant,
    pub memory_limits: MemoryLimits,
    pub allowed_apis: Vec<String>,
}

impl SandboxContext {
    pub fn new(kind: JsEngineKind, limits: MemoryLimits) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            engine_kind: kind,
            created_at: Instant::now(),
            memory_limits: limits,
            allowed_apis: Vec::new(),
        }
    }

    pub fn with_api(mut self, api: &str) -> Self {
        self.allowed_apis.push(api.to_string());
        self
    }

    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
}

pub struct SandboxPool {
    contexts: Mutex<HashMap<String, SandboxContext>>,
    max_contexts: usize,
    default_limits: MemoryLimits,
}

impl SandboxPool {
    pub fn new(max_contexts: usize) -> Self {
        Self {
            contexts: Mutex::new(HashMap::new()),
            max_contexts,
            default_limits: MemoryLimits::restricted(),
        }
    }

    pub fn with_limits(max_contexts: usize, limits: MemoryLimits) -> Self {
        Self {
            contexts: Mutex::new(HashMap::new()),
            max_contexts,
            default_limits: limits,
        }
    }

    pub fn create_context(&self, kind: JsEngineKind) -> Result<SandboxContext, String> {
        let mut contexts = self.contexts.lock().map_err(|e| e.to_string())?;
        if contexts.len() >= self.max_contexts {
            return Err("Sandbox pool exhausted".to_string());
        }
        let ctx = SandboxContext::new(kind, self.default_limits.clone());
        contexts.insert(ctx.id.clone(), ctx.clone());
        Ok(ctx)
    }

    pub fn create_engine(&self, kind: JsEngineKind) -> Result<(SandboxContext, EngineRuntime), EngineError> {
        let ctx = self.create_context(kind)?;
        let engine = EngineRuntime::new(kind)?;
        Ok((ctx, engine))
    }

    pub fn remove_context(&self, id: &str) -> Option<SandboxContext> {
        self.contexts.lock().ok()?.remove(id)
    }

    pub fn context_count(&self) -> usize {
        self.contexts.lock().map(|c| c.len()).unwrap_or(0)
    }

    pub fn cleanup_stale(&self, max_age: std::time::Duration) -> usize {
        let mut contexts = self.contexts.lock().unwrap();
        let before = contexts.len();
        contexts.retain(|_, ctx| ctx.age() < max_age);
        before - contexts.len()
    }

    pub fn get_context(&self, id: &str) -> Option<SandboxContext> {
        self.contexts.lock().ok()?.get(id).cloned()
    }
}

impl Default for SandboxPool {
    fn default() -> Self {
        Self::new(128)
    }
}
