//! Module loading for Jsc

use crate::error::jscError;

pub struct JscModuleLoader;

impl JscModuleLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load(&self, _path: &str) -> Result<String, jscError> {
        Ok(String::new())
    }

    pub fn resolve(&self, _specifier: &str, _base: &str) -> Result<String, jscError> {
        Ok(String::new())
    }
}
