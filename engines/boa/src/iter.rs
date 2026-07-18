use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string};
use boa_engine::property::Attribute;

pub struct IterOps;

impl IterOps {
    pub fn iterator_next(iter: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        let next_fn = if let Some(obj) = iter.as_object() {
            obj.get(js_string!("next"), context)?
        } else {
            return Err(boa_engine::JsError::from_native(
                boa_engine::JsNativeError::typ().with_message("not an iterator")
            ));
        };
        if let Some(func) = next_fn.as_object().filter(|o| o.is_callable()) {
            return func.call(iter, &[], context);
        }
        Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::typ().with_message("next not callable")
        ))
    }

    pub fn collect_iterator(iter: &JsValue, context: &mut Context) -> JsResult<Vec<JsValue>> {
        let mut items = Vec::new();
        loop {
            let result = Self::iterator_next(iter, context)?;
            let done = result.as_object()
                .and_then(|o| o.get(js_string!("done"), context).ok())
                .and_then(|v| v.as_boolean())
                .unwrap_or(false);
            if done { break; }
            let value = result.as_object()
                .and_then(|o| o.get(js_string!("value"), context).ok())
                .unwrap_or(JsValue::undefined());
            items.push(value);
        }
        Ok(items)
    }

    pub fn to_array(value: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        let code = "Array.from(arguments[0])";
        let result = context.eval(boa_engine::Source::from_bytes(code))?;
        if let Some(obj) = result.as_object().filter(|o| o.is_callable()) {
            return obj.call(&JsValue::undefined(), &[value.clone()], context);
        }
        Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::typ().with_message("Array.from not callable")
        ))
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(iterable) = args.first() {
                    Self::to_array(iterable, ctx)
                } else {
                    ctx.eval(boa_engine::Source::from_bytes("[]"))
                }
            }),
            js_string!("toArray"),
            1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let iterable = args.first().cloned().unwrap_or(JsValue::undefined());
                let code = "typeof arguments[0]?.[Symbol.iterator] === 'function'";
                if let Ok(func) = ctx.eval(boa_engine::Source::from_bytes(code)) {
                    if let Some(obj) = func.as_object().filter(|o| o.is_callable()) {
                        return obj.call(&JsValue::undefined(), &[iterable], ctx);
                    }
                }
                Ok(JsValue::from(false))
            }),
            js_string!("isIterable"),
            1usize,
        );
        let obj = builder.build();
        context.register_global_property(
            js_string!("IterOps"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}
