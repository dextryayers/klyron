#[cfg(feature = "native")]
use crate::ffi;
use crate::JSCEngine;

pub struct JSCConsole;

impl JSCConsole {
    pub fn new() -> Self {
        Self
    }

    pub fn log(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn warn(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn error(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn info(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn debug(&self, _engine: &JSCEngine, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn table(&self, _engine: &JSCEngine, _data: *const std::ffi::c_void) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn assert(&self, _engine: &JSCEngine, _condition: bool, _args: &[*const std::ffi::c_void]) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn count(&self, _engine: &JSCEngine, _label: Option<&str>) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn time(&self, _engine: &JSCEngine, _label: Option<&str>) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn time_end(&self, _engine: &JSCEngine, _label: Option<&str>) {
        #[cfg(feature = "native")]
        {}
    }

    pub fn trace(&self, _engine: &JSCEngine) {
        #[cfg(feature = "native")]
        {}
    }
}

impl Default for JSCConsole {
    fn default() -> Self {
        Self::new()
    }
}
