use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string, JsString};
use boa_engine::property::Attribute;
use std::time::{SystemTime, UNIX_EPOCH};

fn random_byte() -> u8 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    ((now.as_nanos() & 0xFF) as u8) ^ (std::process::id() as u8)
}

fn pseudo_random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(len);
    for _ in 0..len {
        bytes.push(random_byte());
    }
    bytes
}

fn random_uuid() -> String {
    let bytes = pseudo_random_bytes(16);
    let mut uuid = String::with_capacity(36);
    for (i, b) in bytes.iter().enumerate() {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            uuid.push('-');
        }
        uuid.push_str(&format!("{:02x}", b));
    }
    uuid
}

fn crypto_random_uuid(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::from(JsString::from(random_uuid())))
}

fn crypto_random_int(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let min = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as u64;
    let max = args.get(1).and_then(|v| v.as_number()).unwrap_or(100.0) as u64;
    if min >= max {
        return Ok(JsValue::from(min as f64));
    }
    let bytes = pseudo_random_bytes(8);
    let val = u64::from_ne_bytes(bytes[..8].try_into().unwrap_or([0u8; 8]));
    let range = max - min;
    Ok(JsValue::from((min + (val % range)) as f64))
}

fn crypto_get_random_values(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(arr) = args.first().and_then(|v| v.as_object()) {
        let len = arr.get(js_string!("length"), context)
            .ok().and_then(|v| v.as_number().map(|n| n as usize)).unwrap_or(0);
        let bytes = pseudo_random_bytes(len);
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
    let data = args.first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();
    let hash = simple_hash(data.as_bytes());
    let hex = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    Ok(JsValue::from(JsString::from(hex)))
}

fn simple_hash(data: &[u8]) -> Vec<u8> {
    let mut hash: Vec<u8> = (0..32).map(|i| (i as u8).wrapping_mul(0x9e)).collect();
    for (i, &b) in data.iter().enumerate() {
        hash[i % 32] = hash[i % 32].wrapping_add(b).wrapping_mul(0x1b).rotate_left(3);
        hash[(i + 1) % 32] ^= b;
    }
    hash
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
