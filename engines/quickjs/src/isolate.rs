use crate::error::QuickJSError;

pub struct QuickJSIsolate {
    rt: Option<*mut std::ffi::c_void>,
    ctx: Option<*mut std::ffi::c_void>,
}

unsafe impl Send for QuickJSIsolate {}
unsafe impl Sync for QuickJSIsolate {}

impl QuickJSIsolate {
    pub fn new() -> Self {
        Self { rt: None, ctx: None }
    }

    pub fn create_isolate() -> Result<Self, QuickJSError> {
        Ok(Self { rt: None, ctx: None })
    }

    pub fn create_context(&mut self) -> Result<(), QuickJSError> {
        Ok(())
    }

    pub fn destroy_context(&mut self) {
        self.rt = None;
        self.ctx = None;
    }

    pub fn with_ctx<F, T>(&self, f: F) -> Result<T, QuickJSError>
    where
        F: FnOnce(*mut std::ffi::c_void) -> Result<T, QuickJSError>,
    {
        let ctx = self.ctx.unwrap_or(std::ptr::null_mut());
        f(ctx)
    }

    pub fn is_initialized(&self) -> bool {
        true
    }
}
