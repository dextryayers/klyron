use crate::error::QuickJSError;

pub struct QuickJSRuntime {
    rt: *mut std::ffi::c_void,
    ctx: *mut std::ffi::c_void,
}

unsafe impl Send for QuickJSRuntime {}
unsafe impl Sync for QuickJSRuntime {}

impl QuickJSRuntime {
    pub fn new() -> Result<Self, QuickJSError> {
        Ok(Self {
            rt: std::ptr::null_mut(),
            ctx: std::ptr::null_mut(),
        })
    }

    pub fn eval(&self, code: &str) -> Result<String, QuickJSError> {
        if code.trim().is_empty() {
            return Ok(String::new());
        }
        Err(QuickJSError::ExecutionFailed(
            "QuickJS native engine not linked; enable the 'quickjs_native' feature".into()
        ))
    }

    pub fn execute_script(&self, _filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.eval(source)
    }

    pub fn execute_module(&self, _filename: &str, source: &str) -> Result<String, QuickJSError> {
        self.eval(source)
    }

    pub fn ctx(&self) -> *mut std::ffi::c_void {
        self.ctx
    }

    pub fn rt(&self) -> *mut std::ffi::c_void {
        self.rt
    }
}

impl Drop for QuickJSRuntime {
    fn drop(&mut self) {
    }
}
