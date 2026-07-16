use crate::runtime::QuickJSRuntime;
use crate::error::QuickJSError;

pub struct QuickJSBindingEngine {
    runtime: QuickJSRuntime,
}

impl QuickJSBindingEngine {
    pub fn new() -> Result<Self, QuickJSError> {
        Ok(Self { runtime: QuickJSRuntime::new()? })
    }

    pub fn eval(&self, code: &str) -> Result<String, QuickJSError> {
        self.runtime.eval(code).map_err(QuickJSError::EvalError)
    }
}

pub type QuickJSEngine = QuickJSBindingEngine;
pub type QuickJSContext = QuickJSRuntime;