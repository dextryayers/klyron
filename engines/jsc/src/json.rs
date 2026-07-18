#[cfg(feature = "native")]
use crate::ffi;
pub struct JSCJson {
    #[cfg(feature = "native")]
    engine: *mut ffi::JSCEngineHandle,
}

impl JSCJson {
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

    #[cfg(feature = "native")]
    pub fn stringify(&self, engine: &crate::ffi::JSCEnginePtr, value: *mut ffi::JSCValueHandle) -> Result<String, String> {
        engine.json_stringify(value)
    }

    #[cfg(feature = "native")]
    pub fn parse(&self, engine: &crate::ffi::JSCEnginePtr, json: &str) -> Result<*mut ffi::JSCValueHandle, String> {
        engine.json_parse(json)
    }

    pub fn serialize<T: serde::Serialize>(&self, _value: &T) -> Result<String, String> {
        serde_json::to_string(_value).map_err(|e| e.to_string())
    }

    pub fn deserialize<'de, T: serde::Deserialize<'de>>(&self, _json: &'de str) -> Result<T, String> {
        serde_json::from_str(_json).map_err(|e| e.to_string())
    }
}

impl Default for JSCJson {
    fn default() -> Self {
        Self::new()
    }
}
