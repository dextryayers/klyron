#[cfg(feature = "native")]
use crate::ffi;

pub struct JSCInspector {
    #[cfg(feature = "native")]
    engine: *mut ffi::JSCEngineHandle,
}

impl JSCInspector {
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

    pub fn is_attached(&self) -> bool {
        false
    }

    pub fn connect(&self, _host: &str, _port: u16) -> Result<(), String> {
        Err("inspector: not implemented for JSC C API".into())
    }

    pub fn disconnect(&self) {}

    pub fn pause_on_next_statement(&self) {}

    pub fn resume(&self) {}
}

impl Default for JSCInspector {
    fn default() -> Self {
        Self::new()
    }
}
