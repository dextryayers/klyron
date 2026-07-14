//! Module loading for V8

use crate::error::v8Error;

pub struct V8ModuleLoader;

impl V8ModuleLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load(&self, _path: &str) -> Result<String, v8Error> {
        Ok(String::new())
    }

    pub fn resolve(&self, _specifier: &str, _base: &str) -> Result<String, v8Error> {
        Ok(String::new())
    }
}
