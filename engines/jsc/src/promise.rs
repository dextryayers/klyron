#[cfg(feature = "native")]
use crate::ffi;

#[cfg(feature = "native")]
pub struct JSCPromise {
    inner: *mut ffi::JSCValueHandle,
}

#[cfg(feature = "native")]
impl JSCPromise {
    pub fn new(engine: &crate::ffi::JSCEnginePtr) -> Result<Self, String> {
        let ptr = engine.promise_new()?;
        Ok(Self { inner: ptr })
    }

    pub fn resolve(&self, engine: &crate::ffi::JSCEnginePtr, value: *mut ffi::JSCValueHandle) -> Result<(), String> {
        engine.promise_resolve(self.inner, value)
    }

    pub fn reject(&self, engine: &crate::ffi::JSCEnginePtr, reason: &str) -> Result<(), String> {
        engine.promise_reject(self.inner, reason)
    }

    pub fn handle(&self) -> *mut ffi::JSCValueHandle {
        self.inner
    }
}

#[cfg(feature = "native")]
impl Drop for JSCPromise {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::klyron_jsc_value_dispose(self.inner) }
        }
    }
}
