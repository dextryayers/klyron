//! Module loading for Boa

use crate::error::boaError;

pub struct BoaModuleLoader;

impl BoaModuleLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load(&self, _path: &str) -> Result<String, boaError> {
        Ok(String::new())
    }

    pub fn resolve(&self, _specifier: &str, _base: &str) -> Result<String, boaError> {
        Ok(String::new())
    }
}
