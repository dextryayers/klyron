//! JS value conversion for Boa

use crate::error::boaError;

#[derive(Debug, Clone)]
pub enum BoaValue {
    Null,
    Undefined,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Array(Vec<BoaValue>),
    Object(std::collections::HashMap<String, BoaValue>),
}

impl BoaValue {
    pub fn from_json(json: &serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Boolean(*b),
            serde_json::Value::Number(n) => n.as_i64()
                .map(Self::Integer)
                .unwrap_or_else(|| Self::Number(n.as_f64().unwrap_or(0.0))),
            serde_json::Value::String(s) => Self::String(s.clone()),
            serde_json::Value::Array(arr) => Self::Array(arr.iter().map(Self::from_json).collect()),
            serde_json::Value::Object(obj) => Self::Object(
                obj.iter().map(|(k,v)| (k.clone(), Self::from_json(v))).collect()
            ),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Undefined => serde_json::Value::Null,
            Self::Boolean(b) => serde_json::Value::Bool(*b),
            Self::Integer(i) => serde_json::json!(i),
            Self::Number(n) => serde_json::json!(n),
            Self::String(s) => serde_json::Value::String(s.clone()),
            Self::Array(arr) => serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect()),
            Self::Object(map) => serde_json::Value::Object(
                map.iter().map(|(k,v)| (k.clone(), v.to_json())).collect()
            ),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null | Self::Undefined => false,
            Self::Boolean(b) => *b,
            Self::Integer(i) => *i != 0,
            Self::Number(n) => *n != 0.0 && !n.is_nan(),
            Self::String(s) => !s.is_empty(),
            Self::Array(a) => !a.is_empty(),
            Self::Object(_) => true,
        }
    }
}

impl From<String> for BoaValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<i64> for BoaValue {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<f64> for BoaValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for BoaValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}
