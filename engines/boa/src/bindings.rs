use crate::error::BoaError;
use boa_engine::{Context, Source};

pub struct BoaBindings;

impl BoaBindings {
    pub fn new() -> Self {
        Self
    }

    pub fn register_bindings(&self, context: &mut Context) -> Result<(), BoaError> {
        context.eval(Source::from_bytes(
            b"globalThis.setTimeout = function(fn, ms) { return 0; };"
        )).map_err(|e| BoaError::ExecutionFailed(e.to_string()))?;

        context.eval(Source::from_bytes(
            b"globalThis.setInterval = function(fn, ms) { return 0; };"
        )).map_err(|e| BoaError::ExecutionFailed(e.to_string()))?;

        Ok(())
    }
}

pub fn register_bindings() -> Vec<&'static str> {
    vec!["console", "timers"]
}

pub fn get_native_binding(_name: &str) -> Option<fn() -> String> {
    None
}
