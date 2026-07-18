use std::collections::HashMap;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NapiError {
    #[error("N-API error: {0}")]
    NapiError(String),
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("Unsupported N-API version: {0} (supported: {NAPI_VERSION_MIN}-{NAPI_VERSION_MAX})")]
    UnsupportedVersion(u32),
    #[error("Incompatible N-API version")]
    IncompatibleVersion(u32),
    #[error("Buffer overflow: {0}")]
    BufferOverflow(String),
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Async work error: {0}")]
    AsyncWorkError(String),
    #[error("Load error: {0}")]
    LoadError(String),
}

pub const NAPI_VERSION_MIN: u32 = 1;
pub const NAPI_VERSION_MAX: u32 = 9;
pub const NAPI_VERSION_CURRENT: u32 = 9;

#[derive(Debug, Clone)]
pub enum NapiValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    Int(i32),
    Uint(u32),
    String(String),
    Object(HashMap<String, NapiValue>),
    Array(Vec<NapiValue>),
    Buffer(Vec<u8>),
    TypedArray(TypedArrayKind, Vec<u8>),
    Function(String),
    External(usize),
    Symbol(String),
}

impl NapiValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            NapiValue::Undefined => "undefined",
            NapiValue::Null => "null",
            NapiValue::Bool(_) => "boolean",
            NapiValue::Number(_) => "number",
            NapiValue::Int(_) => "number",
            NapiValue::Uint(_) => "number",
            NapiValue::String(_) => "string",
            NapiValue::Object(_) => "object",
            NapiValue::Array(_) => "array",
            NapiValue::Buffer(_) => "buffer",
            NapiValue::TypedArray(_, _) => "typedarray",
            NapiValue::Function(_) => "function",
            NapiValue::External(_) => "external",
            NapiValue::Symbol(_) => "symbol",
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, NapiValue::String(_))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            NapiValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            NapiValue::Number(n) => Some(*n),
            NapiValue::Int(i) => Some(*i as f64),
            NapiValue::Uint(u) => Some(*u as f64),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            NapiValue::Int(i) => Some(*i),
            NapiValue::Number(n) => Some(*n as i32),
            NapiValue::Uint(u) => Some(*u as i32),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            NapiValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, NapiValue>> {
        match self {
            NapiValue::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<NapiValue>> {
        match self {
            NapiValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_buffer(&self) -> Option<&[u8]> {
        match self {
            NapiValue::Buffer(buf) => Some(buf.as_slice()),
            _ => None,
        }
    }
}

impl From<bool> for NapiValue {
    fn from(v: bool) -> Self {
        NapiValue::Bool(v)
    }
}

impl From<i32> for NapiValue {
    fn from(v: i32) -> Self {
        NapiValue::Int(v)
    }
}

impl From<u32> for NapiValue {
    fn from(v: u32) -> Self {
        NapiValue::Uint(v)
    }
}

impl From<f64> for NapiValue {
    fn from(v: f64) -> Self {
        NapiValue::Number(v)
    }
}

impl From<String> for NapiValue {
    fn from(v: String) -> Self {
        NapiValue::String(v)
    }
}

impl From<&str> for NapiValue {
    fn from(v: &str) -> Self {
        NapiValue::String(v.to_string())
    }
}

impl From<Vec<u8>> for NapiValue {
    fn from(v: Vec<u8>) -> Self {
        NapiValue::Buffer(v)
    }
}

impl From<Vec<NapiValue>> for NapiValue {
    fn from(v: Vec<NapiValue>) -> Self {
        NapiValue::Array(v)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedArrayKind {
    Int8Array,
    Uint8Array,
    Uint8ClampedArray,
    Int16Array,
    Uint16Array,
    Int32Array,
    Uint32Array,
    Float32Array,
    Float64Array,
    BigInt64Array,
    BigUint64Array,
}

impl TypedArrayKind {
    pub fn element_size(&self) -> usize {
        match self {
            Self::Int8Array | Self::Uint8Array | Self::Uint8ClampedArray => 1,
            Self::Int16Array | Self::Uint16Array => 2,
            Self::Int32Array | Self::Uint32Array | Self::Float32Array => 4,
            Self::Float64Array | Self::BigInt64Array | Self::BigUint64Array => 8,
        }
    }

    pub fn from_str(name: &str) -> Option<Self> {
        match name {
            "Int8Array" => Some(Self::Int8Array),
            "Uint8Array" => Some(Self::Uint8Array),
            "Uint8ClampedArray" => Some(Self::Uint8ClampedArray),
            "Int16Array" => Some(Self::Int16Array),
            "Uint16Array" => Some(Self::Uint16Array),
            "Int32Array" => Some(Self::Int32Array),
            "Uint32Array" => Some(Self::Uint32Array),
            "Float32Array" => Some(Self::Float32Array),
            "Float64Array" => Some(Self::Float64Array),
            "BigInt64Array" => Some(Self::BigInt64Array),
            "BigUint64Array" => Some(Self::BigUint64Array),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Int8Array => "Int8Array",
            Self::Uint8Array => "Uint8Array",
            Self::Uint8ClampedArray => "Uint8ClampedArray",
            Self::Int16Array => "Int16Array",
            Self::Uint16Array => "Uint16Array",
            Self::Int32Array => "Int32Array",
            Self::Uint32Array => "Uint32Array",
            Self::Float32Array => "Float32Array",
            Self::Float64Array => "Float64Array",
            Self::BigInt64Array => "BigInt64Array",
            Self::BigUint64Array => "BigUint64Array",
        }
    }
}

pub fn check_buffer_bounds(buffer: &[u8], offset: usize, length: usize) -> Result<(), NapiError> {
    if offset > buffer.len() {
        return Err(NapiError::BufferOverflow(format!(
            "Offset {offset} exceeds buffer length {}",
            buffer.len()
        )));
    }
    if offset + length > buffer.len() {
        return Err(NapiError::BufferOverflow(format!(
            "Access at offset {offset} for {length} bytes exceeds buffer length {}",
            buffer.len()
        )));
    }
    Ok(())
}

pub fn check_typed_array_bounds(
    kind: TypedArrayKind,
    length: usize,
    byte_offset: usize,
    byte_length: usize,
) -> Result<(), NapiError> {
    let elem_size = kind.element_size();
    let total_bytes = length
        .checked_mul(elem_size)
        .ok_or_else(|| NapiError::BufferOverflow("Integer overflow".into()))?;

    if byte_offset > total_bytes {
        return Err(NapiError::BufferOverflow(format!(
            "Byte offset {byte_offset} exceeds TypedArray byte length {total_bytes}"
        )));
    }
    if byte_offset + byte_length > total_bytes {
        return Err(NapiError::BufferOverflow(format!(
            "Byte access at offset {byte_offset} for {byte_length} bytes exceeds byte length {total_bytes}"
        )));
    }
    Ok(())
}
