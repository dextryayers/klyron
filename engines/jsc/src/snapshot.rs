#[cfg(feature = "native")]
use crate::ffi;

pub struct JSCSnapshot {
    data: Vec<u8>,
}

impl JSCSnapshot {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self { data: bytes.to_vec() }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[cfg(feature = "native")]
    pub fn capture(&mut self, engine: &crate::ffi::JSCEnginePtr) -> Result<(), String> {
        let heap_stats = engine.get_heap_stats()?;
        self.data = format!("{:?}", heap_stats).into_bytes();
        Ok(())
    }

    #[cfg(feature = "native")]
    pub fn restore(&self) -> Result<(), String> {
        Ok(())
    }
}

impl Default for JSCSnapshot {
    fn default() -> Self {
        Self::new()
    }
}
