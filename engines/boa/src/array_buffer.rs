use boa_engine::{Context, JsValue, JsResult, js_string, NativeFunction};

pub struct ArrayBufferWrapper;

impl ArrayBufferWrapper {
    pub fn new(size: usize, context: &mut Context) -> JsResult<JsValue> {
        let code = format!("new ArrayBuffer({})", size);
        context.eval(boa_engine::Source::from_bytes(&code))
    }

    pub fn byte_length(buffer: &JsValue, context: &mut Context) -> JsResult<usize> {
        if let Some(obj) = buffer.as_object() {
            let len = obj.get(js_string!("byteLength"), context)?;
            return Ok(len.as_number().unwrap_or(0.0) as usize);
        }
        Ok(0)
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let size = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as u32;
                Self::new(size as usize, ctx)
            }),
            js_string!("alloc"),
            1usize,
        );
        let obj = builder.build();
        context.register_global_property(
            js_string!("ArrayBufferUtils"),
            obj,
            boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE | boa_engine::property::Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}
