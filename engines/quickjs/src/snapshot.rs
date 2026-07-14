use crate::error::QuickJSError;
use crate::runtime::QuickJSRuntime;
use std::time::SystemTime;

pub struct QuickJSSnapshot {
    data: Vec<u8>,
    created_at: SystemTime,
}

impl QuickJSSnapshot {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, created_at: SystemTime::now() }
    }

    pub fn create(_runtime: &QuickJSRuntime) -> Result<Self, QuickJSError> {
        Ok(Self {
            data: Vec::new(),
            created_at: SystemTime::now(),
        })
    }

    pub fn load(_data: &[u8]) -> Result<QuickJSRuntime, QuickJSError> {
        QuickJSRuntime::new()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }
}
