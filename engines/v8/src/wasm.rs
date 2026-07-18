
#[cfg(feature = "native")]
use crate::error::V8Error;
#[cfg(feature = "native")]
use crate::ffi;

pub struct V8Wasm {
    #[cfg(feature = "native")]
    ctx: *mut ffi::V8ContextHandle,
}

impl V8Wasm {
    #[cfg(feature = "native")]
    pub fn new(ctx: *mut ffi::V8ContextHandle) -> Self {
        Self { ctx }
    }

    #[cfg(not(feature = "native"))]
    pub fn new(_ctx: *mut std::ffi::c_void) -> Self {
        Self {}
    }

    #[cfg(feature = "native")]
    pub fn compile(&self, wasm_bytes: &[u8]) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let ptr = unsafe { ffi::klyron_v8_wasm_compile(self.ctx, wasm_bytes.as_ptr(), wasm_bytes.len()) };
        if ptr.is_null() { Err(V8Error::EvalFailed("wasm compile failed".into())) }
        else { Ok(ptr) }
    }

    #[cfg(feature = "native")]
    pub fn instantiate(&self, wasm_bytes: &[u8], imports: Option<*mut ffi::V8ValueHandle>) -> Result<*mut ffi::V8ValueHandle, V8Error> {
        let ptr = unsafe {
            ffi::klyron_v8_wasm_instantiate(self.ctx, wasm_bytes.as_ptr(), wasm_bytes.len(),
                                               imports.unwrap_or(std::ptr::null_mut()))
        };
        if ptr.is_null() { Err(V8Error::EvalFailed("wasm instantiate failed".into())) }
        else { Ok(ptr) }
    }
}
