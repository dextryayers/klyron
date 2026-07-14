use crate::error::JSCError;

pub struct JSCRuntime {
    ctx: *mut std::ffi::c_void,
}

unsafe impl Send for JSCRuntime {}
unsafe impl Sync for JSCRuntime {}

impl JSCRuntime {
    pub fn new() -> Self {
        Self {
            ctx: std::ptr::null_mut(),
        }
    }

    pub fn eval(&self, code: &str) -> Result<String, JSCError> {
        if code.trim().is_empty() {
            return Ok(String::new());
        }
        Err(JSCError::ExecutionFailed(
            "JSC native engine not linked; enable the 'jsc_native' feature".into()
        ))
    }

    pub fn execute_script(&self, _filename: &str, source: &str) -> Result<String, JSCError> {
        self.eval(source)
    }

    pub fn execute_module(&self, _filename: &str, source: &str) -> Result<String, JSCError> {
        self.eval(source)
    }

    pub fn context(&self) -> *mut std::ffi::c_void {
        self.ctx
    }

    pub fn set_exception_handler(&self) {
    }
}

impl Default for JSCRuntime {
    fn default() -> Self {
        Self::new()
    }
}
