use boa_engine::{Context, JsValue, JsResult, js_string, NativeFunction};

pub struct AsyncUtils;

impl AsyncUtils {
    pub fn resolve(val: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        let s = val.to_string(context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_else(|_| "undefined".to_string());
        let code = format!("Promise.resolve({})", s);
        context.eval(boa_engine::Source::from_bytes(&code))
    }

    pub fn reject(val: &JsValue, context: &mut Context) -> JsResult<JsValue> {
        let msg = val.to_string(context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_else(|_| "error".to_string());
        let code = format!("Promise.reject(new Error({}))", serde_json::to_string(&msg).unwrap_or_default());
        context.eval(boa_engine::Source::from_bytes(&code))
    }

    pub fn all(values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let arr = values.iter().map(|v| {
            v.to_string(context)
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_else(|_| "undefined".to_string())
        }).collect::<Vec<_>>().join(",");
        let code = format!("Promise.all([{}])", arr);
        context.eval(boa_engine::Source::from_bytes(&code))
    }

    pub fn race(values: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let arr = values.iter().map(|v| {
            v.to_string(context)
                .map(|s| s.to_std_string_escaped())
                .unwrap_or_else(|_| "undefined".to_string())
        }).collect::<Vec<_>>().join(",");
        let code = format!("Promise.race([{}])", arr);
        context.eval(boa_engine::Source::from_bytes(&code))
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(val) = args.first() {
                    Self::resolve(val, ctx)
                } else {
                    Self::resolve(&JsValue::undefined(), ctx)
                }
            }),
            js_string!("resolve"),
            1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(val) = args.first() {
                    Self::reject(val, ctx)
                } else {
                    Self::reject(&JsValue::undefined(), ctx)
                }
            }),
            js_string!("reject"),
            1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                match args.get(0) {
                    Some(arr_val) => {
                        let arr_str = arr_val.to_string(ctx)
                            .map(|s| s.to_std_string_escaped())
                            .unwrap_or_else(|_| "[]".to_string());
                        let code = format!("Promise.all({})", arr_str);
                        ctx.eval(boa_engine::Source::from_bytes(&code))
                    }
                    None => Self::resolve(&JsValue::undefined(), ctx),
                }
            }),
            js_string!("all"),
            1usize,
        );
        let obj = builder.build();
        context.register_global_property(
            js_string!("AsyncUtils"),
            obj,
            boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE | boa_engine::property::Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}

pub fn await_promise(promise: &JsValue, _context: &mut Context) -> JsResult<JsValue> {
    Ok(promise.clone())
}

pub fn process_pending(_context: &mut Context) -> JsResult<()> {
    let _ = _context.eval(boa_engine::Source::from_bytes("void 0"))?;
    Ok(())
}
