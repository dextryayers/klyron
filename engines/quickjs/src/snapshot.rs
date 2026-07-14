//! Snapshot support for Quickjs

use crate::error::quickjsError;

pub struct QuickjsSnapshot {
    data: Vec<u8>,
    created_at: std::time::SystemTime,
}

impl QuickjsSnapshot {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, created_at: std::time::SystemTime::now() }
    }

    pub fn create(runtime: &crate::runtime::QuickjsRuntime) -> Result<Self, quickjsError> {
        if !runtime.is_initialized() {
            return Err(quickjsError::NotInitialized);
        }
        Ok(Self {
            data: vec![],
            created_at: std::time::SystemTime::now(),
        })
    }

    pub fn load(data: &[u8]) -> Result<crate::runtime::QuickjsRuntime, quickjsError> {
        let runtime = crate::runtime::QuickjsRuntime::new();
        Ok(runtime)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
