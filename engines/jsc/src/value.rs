#[cfg(feature = "native")]
use crate::ffi;

pub enum JSCValueType {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Object,
    Array,
    Function,
    Error,
    Symbol,
    TypedArray,
}

#[cfg(feature = "native")]
pub struct JSCValue {
    inner: *mut ffi::JSCValueHandle,
}

#[cfg(feature = "native")]
impl JSCValue {
    pub fn new(inner: *mut ffi::JSCValueHandle) -> Option<Self> {
        if inner.is_null() { None } else { Some(Self { inner }) }
    }

    pub fn handle(&self) -> *mut ffi::JSCValueHandle {
        self.inner
    }

    pub fn to_string(&self, engine: &crate::ffi::JSCEnginePtr) -> Result<String, String> {
        engine.value_to_string(self.inner)
    }

    pub fn to_number(&self, engine: &crate::ffi::JSCEnginePtr) -> f64 {
        engine.value_to_number(self.inner)
    }

    pub fn to_bool(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_to_bool(self.inner)
    }

    pub fn is_array(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_array(self.inner)
    }

    pub fn is_function(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_function(self.inner)
    }

    pub fn is_object(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_object(self.inner)
    }

    pub fn is_error(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_error(self.inner)
    }

    pub fn is_symbol(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_symbol(self.inner)
    }

    pub fn is_promise(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_promise(self.inner)
    }

    pub fn is_typed_array(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_typed_array(self.inner)
    }

    pub fn is_null(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_null(self.inner)
    }

    pub fn is_undefined(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_undefined(self.inner)
    }

    pub fn is_boolean(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_boolean(self.inner)
    }

    pub fn is_number(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_number(self.inner)
    }

    pub fn is_string(&self, engine: &crate::ffi::JSCEnginePtr) -> bool {
        engine.value_is_string(self.inner)
    }
}

#[cfg(feature = "native")]
impl Drop for JSCValue {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { ffi::klyron_jsc_value_dispose(self.inner) }
        }
    }
}
