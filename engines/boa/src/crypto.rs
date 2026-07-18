use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string, JsString};
use boa_engine::property::Attribute;

fn fill_random(buf: &mut [u8]) -> Result<(), String> {
    getrandom::getrandom(buf).map_err(|e| format!("getrandom failed: {}", e))
}

fn crypto_random_uuid(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let uuid = uuid::Uuid::new_v4();
    Ok(JsValue::from(JsString::from(uuid.to_string())))
}

fn crypto_random_int(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let min = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as u64;
    let max = args.get(1).and_then(|v| v.as_number()).unwrap_or(100.0) as u64;
    if min >= max {
        return Ok(JsValue::from(min as f64));
    }
    let mut bytes = [0u8; 8];
    if fill_random(&mut bytes).is_err() {
        return Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message("failed to generate random bytes")
        ));
    }
    let val = u64::from_ne_bytes(bytes);
    let range = max - min;
    Ok(JsValue::from((min + (val % range)) as f64))
}

fn crypto_get_random_values(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(arr) = args.first().and_then(|v| v.as_object()) {
        let len = arr.get(js_string!("length"), context)
            .ok().and_then(|v| v.as_number().map(|n| n as usize)).unwrap_or(0);
        let mut bytes = vec![0u8; len];
        if fill_random(&mut bytes).is_err() {
            return Err(boa_engine::JsError::from_native(
                boa_engine::JsNativeError::error().with_message("failed to generate random bytes")
            ));
        }
        for (i, b) in bytes.iter().enumerate() {
            let _ = arr.set(i as u32, JsValue::from(*b as f64), false, context);
        }
        return Ok(JsValue::from(arr.clone()));
    }
    Err(boa_engine::JsError::from_native(
        boa_engine::JsNativeError::typ().with_message("crypto.getRandomValues: first argument must be a typed array")
    ))
}

fn crypto_digest(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    use sha2::Digest;
    let data = args.first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();
    let mut hasher = sha2::Sha256::new();
    hasher.update(data.as_bytes());
    let hash = hasher.finalize();
    let hex = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    Ok(JsValue::from(JsString::from(hex)))
}

pub struct Crypto;

impl Crypto {
    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(NativeFunction::from_fn_ptr(crypto_random_uuid), js_string!("randomUUID"), 0usize)
            .function(NativeFunction::from_fn_ptr(crypto_random_int), js_string!("randomInt"), 2usize)
            .function(NativeFunction::from_fn_ptr(crypto_get_random_values), js_string!("getRandomValues"), 1usize)
            .function(NativeFunction::from_fn_ptr(crypto_digest), js_string!("digest"), 1usize);
        let crypto_obj = builder.build();

        context.register_global_property(
            js_string!("crypto"),
            crypto_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        Ok(())
    }
}
