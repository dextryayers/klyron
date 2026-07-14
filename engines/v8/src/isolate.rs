use crate::error::V8Error;

pub struct V8Isolate {
    initialized: bool,
}

impl V8Isolate {
    pub fn create_isolate() -> Result<Self, V8Error> {
        Ok(Self { initialized: true })
    }

    pub fn create_context(&mut self) -> Result<(), V8Error> {
        if !self.initialized {
            return Err(V8Error::NotInitialized);
        }
        Ok(())
    }

    pub fn with_scope<F, T>(&self, f: F) -> Result<T, V8Error>
    where
        F: FnOnce(&mut ()) -> Result<T, V8Error>,
    {
        f(&mut ())
    }

    pub fn destroy(&mut self) {
        self.initialized = false;
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
