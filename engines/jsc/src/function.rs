#[cfg(feature = "native")]
use crate::ffi;

pub struct JSCFunction {
    #[cfg(feature = "native")]
    inner: *mut ffi::JSCValueHandle,
    name: String,
}

impl JSCFunction {
    #[cfg(feature = "native")]
    pub fn new(engine: &crate::ffi::JSCEnginePtr, name: &str, user_data: *mut std::ffi::c_void) -> Result<Self, String> {
        let ptr = engine.function_new(Some(name), user_data)?;
        Ok(Self { inner: ptr, name: name.to_string() })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[cfg(feature = "native")]
    pub fn handle(&self) -> *mut ffi::JSCValueHandle {
        self.inner
    }

    #[cfg(feature = "native")]
    pub fn call(&self, engine: &crate::ffi::JSCEnginePtr, this_obj: Option<*mut ffi::JSCValueHandle>, args: &[*mut ffi::JSCValueHandle]) -> Result<*mut ffi::JSCValueHandle, String> {
        engine.call_function(self.inner, this_obj, args)
    }
}

#[cfg(feature = "native")]
impl Drop for JSCFunction {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::klyron_jsc_value_dispose(self.inner) }
        }
    }
}
