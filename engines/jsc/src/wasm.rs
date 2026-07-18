#[cfg(feature = "native")]
use crate::ffi;

pub struct JSCWasm {
    #[cfg(feature = "native")]
    engine: *mut ffi::JSCEngineHandle,
}

impl JSCWasm {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "native")]
            engine: std::ptr::null_mut(),
        }
    }

    #[cfg(feature = "native")]
    pub fn from_handle(handle: *mut ffi::JSCEngineHandle) -> Self {
        Self { engine: handle }
    }

    pub fn compile(&self, _wasm_bytes: &[u8]) -> Result<Vec<u8>, String> {
        Err("wasm: compile requires native JSC C API support".into())
    }

    pub fn instantiate(&self, _wasm_module: &[u8], _imports: &[u8]) -> Result<(), String> {
        Err("wasm: instantiate requires native JSC C API support".into())
    }

    pub fn is_wasm_supported(&self) -> bool {
        false
    }
}

impl Default for JSCWasm {
    fn default() -> Self {
        Self::new()
    }
}
