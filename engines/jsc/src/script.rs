#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;

pub struct JSCScript {
    #[cfg(feature = "native")]
    inner: *mut ffi::JSCScriptHandle,
    source: String,
    filename: String,
}

impl JSCScript {
    pub fn new(source: &str, filename: &str) -> Self {
        Self {
            #[cfg(feature = "native")]
            inner: std::ptr::null_mut(),
            source: source.to_string(),
            filename: filename.to_string(),
        }
    }

    #[cfg(feature = "native")]
    pub fn compile(&mut self, engine: &crate::ffi::JSCEnginePtr) -> Result<(), String> {
        let c_source = CString::new(self.source.as_bytes())
            .map_err(|e| format!("CString error: {e}"))?;
        let c_file = CString::new(self.filename.as_bytes())
            .map_err(|e| format!("CString error: {e}"))?;
        let ptr = unsafe {
            ffi::klyron_jsc_compile(engine.engine_handle(), c_source.as_ptr(), c_file.as_ptr())
        };
        if ptr.is_null() {
            return Err("script compilation failed".into());
        }
        self.inner = ptr;
        Ok(())
    }

    #[cfg(feature = "native")]
    pub fn run(&self, engine: &crate::ffi::JSCEnginePtr) -> Result<String, String> {
        if self.inner.is_null() {
            return Err("script not compiled".into());
        }
        engine.run(self.inner)
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
}

#[cfg(feature = "native")]
impl Drop for JSCScript {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::klyron_jsc_script_dispose(self.inner) }
        }
    }
}
