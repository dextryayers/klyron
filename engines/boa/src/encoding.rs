use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string, JsString};
use boa_engine::property::Attribute;

pub struct Encoding;

impl Encoding {
    pub fn encode_text(text: &str, context: &mut Context) -> JsResult<JsValue> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let code = format!("new Uint8Array({})", len);
        let arr = context.eval(boa_engine::Source::from_bytes(&code))?;
        if let Some(obj) = arr.as_object() {
            for (i, b) in bytes.iter().enumerate() {
                obj.set(i as u32, JsValue::from(*b as f64), false, context)
                    .map_err(|e| boa_engine::JsError::from_native(
                        boa_engine::JsNativeError::typ().with_message(e.to_string())
                    ))?;
            }
        }
        Ok(arr)
    }

    pub fn decode_text(data: &JsValue, context: &mut Context) -> JsResult<String> {
        let len = data.as_object()
            .and_then(|o| o.get(js_string!("length"), context).ok())
            .and_then(|v| v.as_number().map(|n| n as usize))
            .unwrap_or(0);
        let mut bytes = Vec::with_capacity(len);
        if let Some(obj) = data.as_object() {
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

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut encoder_builder = boa_engine::object::ObjectInitializer::new(context);
        encoder_builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let text = args.first()
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_default();
                Self::encode_text(&text, ctx)
            }),
            js_string!("encode"), 1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(data) = args.first() {
                    return Self::decode_text(data, ctx).map(|s| JsValue::from(JsString::from(s)));
                }
                Ok(JsValue::from(JsString::from("")))
            }),
            js_string!("decode"), 1usize,
        );
        let encoder_obj = encoder_builder.build();

        context.register_global_property(
            js_string!("TextEncoder"),
            encoder_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        let mut decoder_builder = boa_engine::object::ObjectInitializer::new(context);
        decoder_builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                if let Some(data) = args.first() {
                    return Self::decode_text(data, ctx).map(|s| JsValue::from(JsString::from(s)));
                }
                Ok(JsValue::from(JsString::from("")))
            }),
            js_string!("decode"), 1usize,
        );
        let decoder_obj = decoder_builder.build();

        context.register_global_property(
            js_string!("TextDecoder"),
            decoder_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        Ok(())
    }
}
