use crate::error::BoaError;
use crate::runtime::BoaRuntime;
use serde::{Deserialize, Serialize};

/// Snapshot support for Boa 0.19.
///
/// Boa 0.19 does not have a native snapshot API (added in later versions).
/// This implementation serializes the global object's enumerable properties
/// as a best-effort snapshot using serde.
///
/// NOTE: This does not capture the full VM state (call stack, active realms, etc.).
/// For full snapshots, upgrade to a newer Boa version.

#[derive(Serialize, Deserialize)]
struct SnapshotData {
    global_vars: std::collections::HashMap<String, serde_json::Value>,
}

pub fn create_snapshot(runtime: &mut BoaRuntime) -> Result<Vec<u8>, BoaError> {
    let ctx = runtime.context_mut();
    let global = ctx.global_object();

    let mut vars = std::collections::HashMap::new();
    if let Ok(own_keys) = global.own_property_keys(ctx) {
        for key in own_keys {
            let name = match &key {
                boa_engine::property::PropertyKey::String(s) => s.to_std_string_escaped(),
                boa_engine::property::PropertyKey::Index(i) => i.get().to_string(),
                boa_engine::property::PropertyKey::Symbol(_) => continue,
            };
            // Skip built-in objects
            if is_builtin(&name) {
                continue;
            }
            if let Ok(val) = global.get(key, ctx) {
                let json_val = jsvalue_to_json(&val, ctx);
                vars.insert(name, json_val);
            }
        }
    }

    let snapshot = SnapshotData { global_vars: vars };
    serde_json::to_vec(&snapshot)
        .map_err(|e| BoaError::SnapshotError(e.to_string()))
}

fn is_builtin(name: &str) -> bool {
    name.starts_with("Object")
        || name.starts_with("Array")
        || name.starts_with("Function")
        || name.starts_with("Promise")
        || name.starts_with("String")
        || name.starts_with("Number")
        || name.starts_with("Boolean")
        || name.starts_with("Symbol")
        || name.starts_with("Math")
        || name.starts_with("JSON")
        || name.starts_with("Reflect")
        || name.starts_with("Proxy")
        || name.starts_with("RegExp")
        || name.starts_with("Date")
        || name.starts_with("Set")
        || name.starts_with("Map")
        || name.starts_with("Weak")
        || name.starts_with("Error")
        || name.starts_with("Int")
        || name.starts_with("Float")
        || name.starts_with("BigInt")
        || name.starts_with("console")
        || name.starts_with("process")
        || name.starts_with("fs")
        || name.starts_with("net")
        || name.starts_with("crypto")
}

fn jsvalue_to_json(val: &boa_engine::JsValue, ctx: &mut boa_engine::Context) -> serde_json::Value {
    if val.is_undefined() || val.is_null() {
        return serde_json::Value::Null;
    }
    if let Some(b) = val.as_boolean() {
        return serde_json::Value::Bool(b);
    }
    if let Some(n) = val.as_number() {
        if let Some(json_n) = serde_json::Number::from_f64(n) {
            return serde_json::Value::Number(json_n);
        }
        return serde_json::Value::String(n.to_string());
    }
    if let Ok(s) = val.to_string(ctx) {
        return serde_json::Value::String(s.to_std_string_escaped());
    }
    serde_json::Value::Null
}

pub fn load_snapshot(data: &[u8]) -> Result<BoaRuntime, BoaError> {
    let snapshot: SnapshotData = serde_json::from_slice(data)
        .map_err(|e| BoaError::SnapshotError(format!("invalid snapshot: {}", e)))?;

    let mut runtime = BoaRuntime::new();
    let ctx = runtime.context_mut();
    for (name, value) in &snapshot.global_vars {
        let js_val = json_to_jsvalue(value, ctx);
        let _ = ctx.global_object().set(
            boa_engine::js_string!(name.as_str()),
            js_val,
            false,
            ctx,
        );
    }
    Ok(runtime)
}

fn json_to_jsvalue(val: &serde_json::Value, _ctx: &mut boa_engine::Context) -> boa_engine::JsValue {
    match val {
        serde_json::Value::Null => boa_engine::JsValue::null(),
        serde_json::Value::Bool(b) => boa_engine::JsValue::from(*b),
        serde_json::Value::Number(n) => {
            n.as_f64()
                .map(boa_engine::JsValue::from)
                .unwrap_or(boa_engine::JsValue::null())
        }
        serde_json::Value::String(s) => {
            boa_engine::JsValue::from(boa_engine::JsString::from(s.as_str()))
        }
        serde_json::Value::Array(arr) => {
            use boa_engine::object::builtins::JsArray;
            let js_arr = JsArray::new(_ctx);
            for item in arr {
                let _ = js_arr.push(json_to_jsvalue(item, _ctx), _ctx);
            }
            boa_engine::JsValue::from(js_arr)
        }
        serde_json::Value::Object(obj) => {
            let js_obj = boa_engine::object::ObjectInitializer::new(_ctx).build();
            for (k, v) in obj {
                let _ = js_obj.set(
                    boa_engine::js_string!(k.as_str()),
                    json_to_jsvalue(v, _ctx),
                    false,
                    _ctx,
                );
            }
            boa_engine::JsValue::from(js_obj)
        }
    }
}

pub fn restore_snapshot(data: &[u8]) -> Result<BoaRuntime, BoaError> {
    load_snapshot(data)
}
