use crate::error::JSCError;

pub struct JSCIsolate {
    ctx: Option<*mut std::ffi::c_void>,
}

unsafe impl Send for JSCIsolate {}
unsafe impl Sync for JSCIsolate {}

impl JSCIsolate {
    pub fn new() -> Self {
        Self { ctx: None }
    }

    pub fn create_isolate() -> Result<Self, JSCError> {
        Ok(Self { ctx: None })
    }

    pub fn create_context(&mut self) -> Result<(), JSCError> {
        if self.ctx.is_none() {
            self.ctx = Some(std::ptr::null_mut());
        }
        Ok(())
    }

    pub fn destroy_context(&mut self) {
        self.ctx = None;
    }

    pub fn with_context<F, T>(&self, f: F) -> Result<T, JSCError>
    where
        F: FnOnce(*mut std::ffi::c_void) -> Result<T, JSCError>,
    {
        let ctx = self.ctx.unwrap_or(std::ptr::null_mut());
        f(ctx)
    }

    pub fn is_initialized(&self) -> bool {
        self.ctx.is_some()
    }
}

impl Default for JSCIsolate {
    fn default() -> Self {
        Self::new()
    }
}
