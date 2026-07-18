use crate::error::BoaError;
use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string};
use boa_engine::property::Attribute;

fn console_log(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let msg = args.iter()
        .map(|a| a.to_string(context).map(|s| s.to_std_string_escaped()).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" ");
    println!("[console.log] {msg}");
    Ok(JsValue::undefined())
}

fn console_error(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let msg = args.iter()
        .map(|a| a.to_string(context).map(|s| s.to_std_string_escaped()).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("[console.error] {msg}");
    Ok(JsValue::undefined())
}

pub struct BoaBindings;

impl BoaBindings {
    pub fn new() -> Self { Self }

    pub fn register_bindings(&self, context: &mut Context) -> Result<(), BoaError> {
        let console_obj = boa_engine::object::ObjectInitializer::new(context)
            .function(NativeFunction::from_fn_ptr(console_log), js_string!("log"), 1)
            .function(NativeFunction::from_fn_ptr(console_error), js_string!("error"), 1)
            .build();

        context.register_global_property(
            js_string!("console"),
            console_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| BoaError::from_js_error(&e))?;

        let set_timeout = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let delay = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0);
            println!("[setTimeout] would delay {delay}ms (stub)");
            Ok(JsValue::from(0))
        });
        context.register_global_builtin_callable(js_string!("setTimeout"), 2, set_timeout)
            .map_err(|e| BoaError::from_js_error(&e))?;

        let set_interval = NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let delay = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0);
            println!("[setInterval] would interval {delay}ms (stub)");
            Ok(JsValue::from(0))
        });
        context.register_global_builtin_callable(js_string!("setInterval"), 2, set_interval)
            .map_err(|e| BoaError::from_js_error(&e))?;

        Ok(())
    }
}

pub fn register_bindings() -> Vec<&'static str> {
    vec!["console", "timers"]
}

pub fn get_native_binding(_name: &str) -> Option<fn() -> String> {
    None
}
