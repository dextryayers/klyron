use crate::error::BoaError;
use boa_engine::{Context, Source};

pub struct BoaRuntime {
    context: Context,
}

impl BoaRuntime {
    pub fn new() -> Self {
        Self {
            context: Context::default(),
        }
    }

    pub fn eval(&mut self, code: &str) -> Result<String, BoaError> {
        let result = self.context.eval(Source::from_bytes(code.as_bytes()))
            .map_err(|e| BoaError::ExecutionFailed(e.to_string()))?;
        Ok(result.to_string(&mut self.context).map(|s| s.to_std_string_escaped()).unwrap_or_default())
    }

    pub fn execute_script(&mut self, _filename: &str, source: &str) -> Result<String, BoaError> {
        self.eval(source)
    }

    pub fn execute_module(&mut self, _filename: &str, source: &str) -> Result<String, BoaError> {
        self.eval(source)
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}

impl Default for BoaRuntime {
    fn default() -> Self {
        Self::new()
    }
}
