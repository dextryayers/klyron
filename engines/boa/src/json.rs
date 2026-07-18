use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string, JsString};
use boa_engine::property::Attribute;

pub struct JsonUtils;

impl JsonUtils {
    pub fn parse_safe(text: &str, context: &mut Context) -> JsResult<JsValue> {
        let sanitized = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
        let code = format!("try {{ JSON.parse({}) }} catch(e) {{ null }}", sanitized);
        context.eval(boa_engine::Source::from_bytes(&code))
    }

    pub fn stringify(value: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        let code = "JSON.stringify(arguments[0])";
        let result = context.eval(boa_engine::Source::from_bytes(code))?;
        if let Some(obj) = result.as_object().filter(|o| o.is_callable()) {
            return obj.call(&JsValue::undefined(), &[value.clone()], context);
        }
        Ok(value.to_string(context)
            .map(|s| JsValue::from(JsString::from(s.to_std_string_escaped())))
            .unwrap_or(JsValue::undefined()))
    }

    pub fn deep_clone(value: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        let code = "JSON.parse(JSON.stringify(arguments[0]))";
        let result = context.eval(boa_engine::Source::from_bytes(code))?;
        if let Some(obj) = result.as_object().filter(|o| o.is_callable()) {
            return obj.call(&JsValue::undefined(), &[value.clone()], context);
        }
        Ok(value.clone())
    }

    pub fn is_valid_json(text: &str, context: &mut Context) -> bool {
        let sanitized = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
        let code = format!("try {{ JSON.parse({}); true }} catch(e) {{ false }}", sanitized);
        if let Ok(val) = context.eval(boa_engine::Source::from_bytes(&code)) {
            return val.as_boolean().unwrap_or(false);
        }
        false
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let text = args.first()
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                Self::parse_safe(&text, ctx)
            }),
            js_string!("parseSafe"), 1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let val = args.first().cloned().unwrap_or(JsValue::undefined());
                Self::stringify(&val, ctx)
            }),
            js_string!("stringify"), 1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let val = args.first().cloned().unwrap_or(JsValue::undefined());
                Self::deep_clone(&val, ctx)
            }),
            js_string!("deepClone"), 1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let text = args.first()
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                Ok(JsValue::from(Self::is_valid_json(&text, ctx)))
            }),
            js_string!("isValid"), 1usize,
        );
        let obj = builder.build();
        context.register_global_property(
            js_string!("JsonUtils"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}
