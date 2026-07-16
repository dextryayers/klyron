use serde_json::Value as JsonValue;

#[derive(Debug, Clone)]
pub enum QuickJSValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<QuickJSValue>),
    Object(Vec<(String, QuickJSValue)>),
    Buffer(Vec<u8>),
}

impl QuickJSValue {
    pub fn from_json(val: &JsonValue) -> Self {
        match val {
            JsonValue::Null => Self::Null,
            JsonValue::Bool(b) => Self::Bool(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() { Self::Int(i) }
                else if let Some(f) = n.as_f64() { Self::Float(f) }
                else { Self::Null }
            }
            JsonValue::String(s) => Self::String(s.clone()),
            JsonValue::Array(arr) => Self::Array(arr.iter().map(Self::from_json).collect()),
            JsonValue::Object(obj) => Self::Object(
                obj.iter().map(|(k, v)| (k.clone(), Self::from_json(v))).collect()
            ),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        match self {
            Self::Null => JsonValue::Null,
            Self::Bool(b) => JsonValue::Bool(*b),
            Self::Int(i) => JsonValue::Number((*i).into()),
            Self::Float(f) => serde_json::json!(f),
            Self::String(s) => JsonValue::String(s.clone()),
            Self::Array(arr) => JsonValue::Array(arr.iter().map(|v| v.to_json()).collect()),
            Self::Object(obj) => JsonValue::Object(
                obj.iter().map(|(k, v)| (k.clone(), v.to_json())).collect()
            ),
            Self::Buffer(b) => JsonValue::Array(b.iter().map(|n| JsonValue::Number((*n).into())).collect()),
        }
    }
}