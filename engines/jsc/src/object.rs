#[cfg(feature = "native")]
use crate::ffi;
use std::collections::HashMap;

pub struct JSCObject {
    properties: HashMap<String, Vec<u8>>,
}

impl JSCObject {
    pub fn new() -> Self {
        Self { properties: HashMap::new() }
    }

    pub fn set_property(&mut self, key: &str, value: Vec<u8>) {
        self.properties.insert(key.to_string(), value);
    }

    pub fn get_property(&self, key: &str) -> Option<&[u8]> {
        self.properties.get(key).map(|v| v.as_slice())
    }

    pub fn delete_property(&mut self, key: &str) -> bool {
        self.properties.remove(key).is_some()
    }

    pub fn has_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    pub fn keys(&self) -> Vec<String> {
        let mut k: Vec<String> = self.properties.keys().cloned().collect();
        k.sort();
        k
    }

    pub fn len(&self) -> usize {
        self.properties.len()
    }

    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }

    pub fn clear(&mut self) {
        self.properties.clear();
    }

    #[cfg(feature = "native")]
    pub fn from_value(engine: &crate::ffi::JSCEnginePtr, value: *mut ffi::JSCValueHandle) -> Result<Self, String> {
        let obj = Self::new();
        Ok(obj)
    }

    #[cfg(feature = "native")]
    pub fn to_value(&self, engine: &crate::ffi::JSCEnginePtr) -> *mut ffi::JSCValueHandle {
        std::ptr::null_mut()
    }
}

impl Default for JSCObject {
    fn default() -> Self {
        Self::new()
    }
}
