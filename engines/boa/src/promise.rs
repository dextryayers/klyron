use crate::error::BoaError;
use boa_engine::{Context, JsValue, JsResult, JsError, JsNativeError};
use boa_engine::object::builtins::JsPromise;

pub fn create_js_promise(context: &mut Context) -> JsResult<JsValue> {
    let (promise, _resolvers) = JsPromise::new_pending(context);
    Ok(JsValue::from(promise))
}

pub fn promise_resolve(val: &JsValue, context: &mut Context) -> Result<JsValue, BoaError> {
    let resolved = JsPromise::resolve(val.clone(), context);
    Ok(JsValue::from(resolved))
}

pub fn promise_reject(val: &JsValue, context: &mut Context) -> Result<JsValue, BoaError> {
    let js_error = JsError::from_native(
        JsNativeError::error().with_message(val.to_string(context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_else(|_| "unknown error".to_string()))
    );
    let rejected = JsPromise::reject(js_error, context);
    Ok(JsValue::from(rejected))
}
