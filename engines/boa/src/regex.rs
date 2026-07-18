use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string, JsString};
use boa_engine::property::Attribute;

pub struct RegexUtils;

impl RegexUtils {
    pub fn test(pattern: &str, flags: &str, text: &str, context: &mut Context) -> JsResult<bool> {
        let re = serde_json::to_string(pattern).unwrap_or_else(|_| "\"\"".to_string());
        let fl = serde_json::to_string(flags).unwrap_or_else(|_| "\"\"".to_string());
        let tx = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
        let code = format!("new RegExp({}, {}).test({})", re, fl, tx);
        let result = context.eval(boa_engine::Source::from_bytes(&code))?;
        Ok(result.as_boolean().unwrap_or(false))
    }

    pub fn exec(pattern: &str, flags: &str, text: &str, context: &mut Context) -> JsResult<JsValue> {
        let re = serde_json::to_string(pattern).unwrap_or_else(|_| "\"\"".to_string());
        let fl = serde_json::to_string(flags).unwrap_or_else(|_| "\"\"".to_string());
        let tx = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
        let code = format!("new RegExp({}, {}).exec({})", re, fl, tx);
        context.eval(boa_engine::Source::from_bytes(&code))
    }

    pub fn escape_regex(text: &str) -> String {
        let special = r"[.*+?^${}()|[\]\\]";
        let mut result = String::with_capacity(text.len());
        for c in text.chars() {
            if special.contains(c) { result.push('\\'); }
            result.push(c);
        }
        result
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let pattern = args.first().and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            let text = args.get(1).and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            let flags = args.get(2).and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            Self::test(&pattern, &flags, &text, ctx).map(|b| JsValue::from(b))
        }), js_string!("test"), 3usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, ctx| {
            let pattern = args.first().and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            let text = args.get(1).and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            let flags = args.get(2).and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            Self::exec(&pattern, &flags, &text, ctx)
        }), js_string!("exec"), 3usize)
        .function(NativeFunction::from_fn_ptr(|_this, args, _ctx| {
            let text = args.first().and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
            Ok(JsValue::from(JsString::from(Self::escape_regex(&text))))
        }), js_string!("escape"), 1usize);
        let obj = builder.build();
        context.register_global_property(
            js_string!("RegexUtils"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}
