use crate::error::JSCError;
use crate::runtime::JSCRuntime;
use std::time::SystemTime;

pub struct JSCSnapshot {
    data: Vec<u8>,
    created_at: SystemTime,
}

impl JSCSnapshot {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, created_at: SystemTime::now() }
    }

    pub fn create(_runtime: &JSCRuntime) -> Result<Self, JSCError> {
        Ok(Self {
            data: Vec::new(),
            created_at: SystemTime::now(),
        })
    }

    pub fn load(_data: &[u8]) -> Result<JSCRuntime, JSCError> {
        Ok(JSCRuntime::new())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }
}
