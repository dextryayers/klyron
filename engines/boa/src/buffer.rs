use boa_engine::{Context, JsValue, JsResult, js_string, NativeFunction, JsString};

pub struct Buffer;

impl Buffer {
    pub fn alloc(size: usize, context: &mut Context) -> JsResult<JsValue> {
        let code = format!("new Uint8Array({})", size);
        context.eval(boa_engine::Source::from_bytes(&code))
    }

    pub fn from_string(text: &str, context: &mut Context) -> JsResult<JsValue> {
        let json = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
        let code = format!("new TextEncoder().encode({})", json);
        let result = context.eval(boa_engine::Source::from_bytes(&code));
        if result.is_ok() {
            return result;
        }
        let escaped = text.bytes().map(|b| b.to_string()).collect::<Vec<_>>().join(",");
        let code = format!("new Uint8Array([{}])", escaped);
        context.eval(boa_engine::Source::from_bytes(&code))
    }

    pub fn to_string(buffer: &JsValue, context: &mut Context) -> JsResult<String> {
        let len = buffer.as_object()
            .and_then(|o| o.get(js_string!("length"), context).ok())
            .and_then(|v| v.as_number().map(|n| n as usize))
            .unwrap_or(0);
        let mut bytes = Vec::with_capacity(len);
        if let Some(obj) = buffer.as_object() {
            for i in 0..len {
                if let Ok(val) = obj.get(i as u32, context) {
                    if let Some(n) = val.as_number() {
                        bytes.push(n as u8);
                    }
                }
            }
        }
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    pub fn concat(buffers: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let total_len: usize = buffers.iter().filter_map(|b| {
            b.as_object().and_then(|o| o.get(js_string!("length"), context).ok())
                .and_then(|v| v.as_number().map(|n| n as usize))
        }).sum();
        let result = Self::alloc(total_len, context)?;
        let mut offset = 0;
        for buf in buffers {
            if let Some(obj) = buf.as_object() {
                let len = obj.get(js_string!("length"), context)
                    .ok().and_then(|v| v.as_number().map(|n| n as u32)).unwrap_or(0);
                for i in 0..len {
                    if let Ok(val) = obj.get(i, context) {
                        if let Some(result_obj) = result.as_object() {
                            let _ = result_obj.set(offset as u32 + i, val, false, context);
                        }
                    }
                }
                offset += len as usize;
            }
        }
        Ok(result)
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let size = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as u32;
                Self::alloc(size as usize, ctx)
            }),
            js_string!("alloc"),
            1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(s) = args.first().and_then(|v| v.as_string()) {
                    let text = s.to_std_string_escaped();
                    return Self::from_string(&text, ctx);
                }
                Self::from_string("", ctx)
            }),
            js_string!("fromString"),
            1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(buf) = args.first() {
                    return Self::to_string(buf, ctx).map(|s| JsValue::from(JsString::from(s)));
                }
                Ok(JsValue::from(JsString::from("")))
            }),
            js_string!("toString"),
            1usize,
        );
        let obj = builder.build();
        context.register_global_property(
            js_string!("Buffer"),
            obj,
            boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE | boa_engine::property::Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}
