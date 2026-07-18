use std::collections::HashMap;
use boa_engine::{Context, JsValue, JsString};
use boa_engine::object::builtins::JsArray;

#[derive(Debug, Clone)]
pub enum BoaValue {
    Null,
    Undefined,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Array(Vec<BoaValue>),
    Object(HashMap<String, BoaValue>),
}

impl BoaValue {
    pub fn from_json(json: &serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Boolean(*b),
            serde_json::Value::Number(n) => {
                n.as_i64().map(Self::Integer)
                    .or_else(|| n.as_f64().map(Self::Number))
                    .unwrap_or(Self::Null)
            }
            serde_json::Value::String(s) => Self::String(s.clone()),
            serde_json::Value::Array(arr) => {
                Self::Array(arr.iter().map(Self::from_json).collect())
            }
            serde_json::Value::Object(obj) => {
                Self::Object(obj.iter()
                    .map(|(k, v)| (k.clone(), Self::from_json(v)))
                    .collect())
            }
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Undefined => serde_json::Value::Null,
            Self::Boolean(b) => serde_json::Value::Bool(*b),
            Self::Integer(i) => serde_json::Value::Number((*i).into()),
            Self::Number(n) => serde_json::Value::Number(
                serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0))
            ),
            Self::String(s) => serde_json::Value::String(s.clone()),
            Self::Array(arr) => serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect()),
            Self::Object(obj) => serde_json::Value::Object(
                obj.iter().map(|(k, v)| (k.clone(), v.to_json())).collect()
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
            Self::Array(_) | Self::Object(_) => true,
        }
    }

    pub fn from_js(js_value: &JsValue, context: &mut Context) -> Self {
        if js_value.is_undefined() { return Self::Undefined; }
        if js_value.is_null() { return Self::Null; }
        if let Some(b) = js_value.as_boolean() { return Self::Boolean(b); }
        if let Some(n) = js_value.as_number() {
            if n.fract() == 0.0 && n.is_finite() { return Self::Integer(n as i64); }
            return Self::Number(n);
        }
        if let Ok(s) = js_value.to_string(context) {
            return Self::String(s.to_std_string_escaped());
        }
        Self::Undefined
    }

    pub fn to_js(&self, context: &mut Context) -> JsValue {
        match self {
            Self::Null => JsValue::null(),
            Self::Undefined => JsValue::undefined(),
            Self::Boolean(b) => JsValue::from(*b),
            Self::Integer(i) => JsValue::from(*i as f64),
            Self::Number(n) => JsValue::from(*n),
            Self::String(s) => {
                let js_str = JsString::from(s.as_str());
                JsValue::from(js_str)
            }
            Self::Array(arr) => {
                let js_arr = JsArray::new(context);
                for (i, v) in arr.iter().enumerate() {
                    let _ = js_arr.set(i as u32, v.to_js(context), false, context);
                }
                js_arr.into()
            }
            Self::Object(obj) => {
                let js_obj = boa_engine::object::ObjectInitializer::new(context).build();
                for (k, v) in obj {
                    let js_key = JsString::from(k.as_str());
                    let _ = js_obj.set(js_key, v.to_js(context), false, context);
                }
                js_obj.into()
            }
        }
    }
}

impl From<String> for BoaValue { fn from(s: String) -> Self { Self::String(s) } }
impl From<i64> for BoaValue { fn from(i: i64) -> Self { Self::Integer(i) } }
impl From<f64> for BoaValue { fn from(n: f64) -> Self { Self::Number(n) } }
impl From<bool> for BoaValue { fn from(b: bool) -> Self { Self::Boolean(b) } }
