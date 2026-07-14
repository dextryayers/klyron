use crate::error::BoaError;
use boa_engine::Context;

pub struct BoaIsolate {
    context: Option<Context>,
}

impl BoaIsolate {
    pub fn new() -> Self {
        Self { context: None }
    }

    pub fn create_isolate() -> Result<Self, BoaError> {
        Ok(Self {
            context: Some(Context::default()),
        })
    }

    pub fn create_context(&mut self) -> Result<(), BoaError> {
        if self.context.is_none() {
            self.context = Some(Context::default());
        }
        Ok(())
    }

    pub fn with_context<F, T>(&mut self, f: F) -> Result<T, BoaError>
    where
        F: FnOnce(&mut Context) -> Result<T, BoaError>,
    {
        let context = self.context.as_mut()
            .ok_or(BoaError::NotInitialized)?;
        f(context)
    }

    pub fn destroy_context(&mut self) {
        self.context = None;
    }

    pub fn is_initialized(&self) -> bool {
        self.context.is_some()
    }
}

impl Default for BoaIsolate {
    fn default() -> Self {
        Self::new()
    }
}
