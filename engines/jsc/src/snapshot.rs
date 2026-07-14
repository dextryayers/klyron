//! Snapshot support for Jsc

use crate::error::jscError;

pub struct JscSnapshot {
    data: Vec<u8>,
    created_at: std::time::SystemTime,
}

impl JscSnapshot {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, created_at: std::time::SystemTime::now() }
    }

    pub fn create(runtime: &crate::runtime::JscRuntime) -> Result<Self, jscError> {
        if !runtime.is_initialized() {
            return Err(jscError::NotInitialized);
        }
        Ok(Self {
            data: vec![],
            created_at: std::time::SystemTime::now(),
        })
    }

    pub fn load(data: &[u8]) -> Result<crate::runtime::JscRuntime, jscError> {
        let runtime = crate::runtime::JscRuntime::new();
        Ok(runtime)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
