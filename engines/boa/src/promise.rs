use crate::error::BoaError;
use boa_engine::{Context, JsValue, JsResult};

pub fn js_promise_new(context: &mut Context) -> JsResult<boa_engine::JsValue> {
    let code = "new Promise((resolve) => resolve(undefined))";
    // In 0.19, we use Source::from_bytes
    Ok(context.eval(boa_engine::Source::from_bytes(code))?)
}

pub fn promise_resolve(val: &JsValue, context: &mut Context) -> Result<String, BoaError> {
    Ok(val.to_string(context)
        .map_err(|e| BoaError::from_js_error(&e))?
        .to_std_string_escaped())
}

pub fn promise_reject(val: &JsValue, context: &mut Context) -> Result<String, BoaError> {
    Ok(val.to_string(context)
        .map_err(|e| BoaError::from_js_error(&e))?
        .to_std_string_escaped())
}

pub fn create_js_promise(context: &mut Context) -> JsResult<JsValue> {
    js_promise_new(context)
}
