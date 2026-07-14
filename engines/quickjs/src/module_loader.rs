//! Module loading for Quickjs

use crate::error::quickjsError;

pub struct QuickjsModuleLoader;

impl QuickjsModuleLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load(&self, _path: &str) -> Result<String, quickjsError> {
        Ok(String::new())
    }

    pub fn resolve(&self, _specifier: &str, _base: &str) -> Result<String, quickjsError> {
        Ok(String::new())
    }
}
