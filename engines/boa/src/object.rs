use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string};
use boa_engine::property::Attribute;

pub struct ObjectUtils;

impl ObjectUtils {
    pub fn get_property(obj: &JsValue, key: &str, context: &mut Context) -> JsResult<JsValue> {
        if let Some(js_obj) = obj.as_object() {
            return js_obj.get(js_string!(key), context);
        }
        Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::typ().with_message("first argument must be an object")
        ))
    }

    pub fn set_property(obj: &JsValue, key: &str, value: JsValue, context: &mut Context) -> JsResult<()> {
        if let Some(js_obj) = obj.as_object() {
            js_obj.set(js_string!(key), value, false, context)
                .map_err(|e| boa_engine::JsError::from_native(
                    boa_engine::JsNativeError::typ().with_message(e.to_string())
                ))?;
            return Ok(());
        }
        Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::typ().with_message("first argument must be an object")
        ))
    }

    pub fn has_property(obj: &JsValue, key: &str, context: &mut Context) -> JsResult<bool> {
        if let Some(_js_obj) = obj.as_object() {
            let code = format!("arguments[0].hasOwnProperty({})", serde_json::to_string(key).unwrap_or_default());
            let result = context.eval(boa_engine::Source::from_bytes(&code))?;
            if let Some(func) = result.as_object().filter(|o| o.is_callable()) {
                let val = func.call(&JsValue::undefined(), &[obj.clone()], context)?;
                return Ok(val.as_boolean().unwrap_or(false));
            }
            return Ok(false);
        }
        Ok(false)
    }

    pub fn delete_property(obj: &JsValue, key: &str, context: &mut Context) -> JsResult<bool> {
        if let Some(js_obj) = obj.as_object() {
            return js_obj.delete_property_or_throw(js_string!(key), context);
        }
        Ok(false)
    }

    pub fn keys(obj: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        let code = "Object.keys(arguments[0])";
        let result = context.eval(boa_engine::Source::from_bytes(code))?;
        if let Some(func) = result.as_object().filter(|o| o.is_callable()) {
            return func.call(&JsValue::undefined(), &[obj.clone()], context);
        }
        Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::typ().with_message("Object.keys not available")
        ))
    }

    pub fn values(obj: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        let code = "Object.values(arguments[0])";
        let result = context.eval(boa_engine::Source::from_bytes(code))?;
        if let Some(func) = result.as_object().filter(|o| o.is_callable()) {
            return func.call(&JsValue::undefined(), &[obj.clone()], context);
        }
        Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::typ().with_message("Object.values not available")
        ))
    }

    pub fn entries(obj: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        let code = "Object.entries(arguments[0])";
        let result = context.eval(boa_engine::Source::from_bytes(code))?;
        if let Some(func) = result.as_object().filter(|o| o.is_callable()) {
            return func.call(&JsValue::undefined(), &[obj.clone()], context);
        }
        Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::typ().with_message("Object.entries not available")
        ))
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let obj = args.first().ok_or_else(|| {
                    boa_engine::JsError::from_native(
                        boa_engine::JsNativeError::typ().with_message("missing argument")
                    )
                })?;
                let key = args.get(1)
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                Self::get_property(obj, &key, ctx)
            }), js_string!("get"), 2usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let obj = args.first().ok_or_else(|| {
                    boa_engine::JsError::from_native(
                        boa_engine::JsNativeError::typ().with_message("missing argument")
                    )
                })?;
                let key = args.get(1)
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                let val = args.get(2).cloned().unwrap_or(JsValue::undefined());
                Self::set_property(obj, &key, val, ctx)?;
                Ok(JsValue::undefined())
            }), js_string!("set"), 3usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let obj = args.first().ok_or_else(|| {
                    boa_engine::JsError::from_native(
                        boa_engine::JsNativeError::typ().with_message("missing argument")
                    )
                })?;
                let key = args.get(1)
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                Ok(JsValue::from(Self::has_property(obj, &key, ctx)?))
            }), js_string!("has"), 2usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let obj = args.first().ok_or_else(|| {
                    boa_engine::JsError::from_native(
                        boa_engine::JsNativeError::typ().with_message("missing argument")
                    )
                })?;
                Self::keys(obj, ctx)
            }), js_string!("keys"), 1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let obj = args.first().ok_or_else(|| {
                    boa_engine::JsError::from_native(
                        boa_engine::JsNativeError::typ().with_message("missing argument")
                    )
                })?;
                Self::values(obj, ctx)
            }), js_string!("values"), 1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let obj = args.first().ok_or_else(|| {
                    boa_engine::JsError::from_native(
                        boa_engine::JsNativeError::typ().with_message("missing argument")
                    )
                })?;
                Self::entries(obj, ctx)
            }), js_string!("entries"), 1usize,
        );
        let obj = builder.build();
        context.register_global_property(
            js_string!("ObjectUtils"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}
