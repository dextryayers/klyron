//! Common types for the Klyron polyglot runtime.
//!
//! Provides `JsonValue`, `KlyronError`, and `Result` types
//! used throughout the SDK modules.

use std::collections::BTreeMap;
use std::fmt;

/// A JSON-compatible value type.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    Some(*n as i64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, JsonValue>> {
        match self {
            JsonValue::Object(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, JsonValue>> {
        match self {
            JsonValue::Object(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }
}

impl From<String> for JsonValue {
    fn from(s: String) -> Self {
        JsonValue::String(s)
    }
}

impl From<&str> for JsonValue {
    fn from(s: &str) -> Self {
        JsonValue::String(s.to_string())
    }
}

impl From<f64> for JsonValue {
    fn from(n: f64) -> Self {
        JsonValue::Number(n)
    }
}

impl From<i64> for JsonValue {
    fn from(n: i64) -> Self {
        JsonValue::Number(n as f64)
    }
}

impl From<bool> for JsonValue {
    fn from(b: bool) -> Self {
        JsonValue::Bool(b)
    }
}

impl From<Vec<JsonValue>> for JsonValue {
    fn from(arr: Vec<JsonValue>) -> Self {
        JsonValue::Array(arr)
    }
}

impl From<BTreeMap<String, JsonValue>> for JsonValue {
    fn from(map: BTreeMap<String, JsonValue>) -> Self {
        JsonValue::Object(map)
    }
}

/// Error type for Klyron operations.
#[derive(Debug)]
pub enum KlyronError {
    Io(std::io::Error),
    Parse(String),
    Http(String),
    Crypto(String),
    Msg(String),
}

impl fmt::Display for KlyronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KlyronError::Io(e) => write!(f, "IO error: {}", e),
            KlyronError::Parse(s) => write!(f, "Parse error: {}", s),
            KlyronError::Http(s) => write!(f, "HTTP error: {}", s),
            KlyronError::Crypto(s) => write!(f, "Crypto error: {}", s),
            KlyronError::Msg(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for KlyronError {}

impl From<std::io::Error> for KlyronError {
    fn from(e: std::io::Error) -> Self {
        KlyronError::Io(e)
    }
}

impl From<String> for KlyronError {
    fn from(s: String) -> Self {
        KlyronError::Msg(s)
    }
}

impl From<&str> for KlyronError {
    fn from(s: &str) -> Self {
        KlyronError::Msg(s.to_string())
    }
}

/// Convenience result type for Klyron operations.
pub type Result<T> = std::result::Result<T, KlyronError>;
