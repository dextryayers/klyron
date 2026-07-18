use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string, JsString};
use boa_engine::property::Attribute;

#[derive(Debug, Clone)]
pub struct Url {
    pub href: String,
    pub protocol: String,
    pub hostname: String,
    pub port: u16,
    pub pathname: String,
    pub search: String,
    pub hash: String,
    pub host: String,
    pub origin: String,
}

impl Url {
    pub fn parse(url_str: &str) -> Result<Self, String> {
        let url_str = url_str.trim();
        if url_str.is_empty() { return Err("empty URL".to_string()); }

        let mut working = url_str.to_string();
        if !working.contains("://") { working = format!("https://{}", working); }

        let protocol_end = working.find("://").ok_or_else(|| "missing protocol".to_string())?;
        let protocol = working[..protocol_end + 1].to_string();
        let after_protocol = &working[protocol_end + 3..];

        let (host_part, rest) = if let Some(pos) = after_protocol.find('/') {
            after_protocol.split_at(pos)
        } else {
            (after_protocol, "")
        };

        let (hostname, port) = if let Some(pos) = host_part.find(':') {
            let host = &host_part[..pos];
            let port_num: u16 = host_part[pos + 1..].parse().unwrap_or(80);
            (host.to_string(), port_num)
        } else {
            (host_part.to_string(), if protocol.starts_with("https") { 443 } else { 80 })
        };

        let path_and_query = if rest.is_empty() { "/" } else { rest };
        let (pathname, search_and_hash) = if let Some(pos) = path_and_query.find('?') {
            path_and_query.split_at(pos)
        } else {
            (path_and_query, "")
        };

        let (search, hash) = if let Some(pos) = search_and_hash.find('#') {
            let (s, h) = search_and_hash.split_at(pos);
            (s.to_string(), h.to_string())
        } else {
            (search_and_hash.to_string(), String::new())
        };

        let host = if port == 80 || port == 443 {
            hostname.clone()
        } else {
            format!("{}:{}", hostname, port)
        };

        let origin = format!("{}//{}", protocol, host);
        let href = format!("{}//{}{}{}{}", protocol, host, pathname, search, hash);

        Ok(Self {
            href, protocol: protocol.trim_end_matches(':').to_string(),
            hostname, port, pathname: pathname.to_string(), search, hash: hash.trim_start_matches('#').to_string(),
            host, origin,
        })
    }

    pub fn resolve(base: &str, relative: &str) -> Result<String, String> {
        let base_url = Self::parse(base)?;
        if relative.starts_with("http://") || relative.starts_with("https://") {
            return Ok(relative.to_string());
        }
        if relative.starts_with("//") {
            return Ok(format!("{}:{}", base_url.protocol, relative));
        }
        if relative.starts_with('/') {
            return Ok(format!("{}//{}{}", base_url.protocol, base_url.host, relative));
        }
        if relative.starts_with('?') {
            return Ok(format!("{}//{}{}{}", base_url.protocol, base_url.host, base_url.pathname, relative));
        }
        if relative.starts_with('#') {
            return Ok(format!("{}//{}{}{}{}", base_url.protocol, base_url.host, base_url.pathname, base_url.search, relative));
        }
        let base_dir = if base_url.pathname.ends_with('/') {
            base_url.pathname.clone()
        } else {
            let last_slash = base_url.pathname.rfind('/').unwrap_or(0);
            base_url.pathname[..=last_slash].to_string()
        };
        Ok(format!("{}//{}{}{}", base_url.protocol, base_url.host, base_dir, relative))
    }
}

fn url_parse_fn(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url_str = args.first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();
    match Url::parse(&url_str) {
        Ok(url) => {
            let code = format!(
                "({{ href: {}, protocol: {}, hostname: {}, port: {}, pathname: {}, search: {}, hash: {}, host: {}, origin: {} }})",
                serde_json::to_string(&url.href).unwrap_or_default(),
                serde_json::to_string(&url.protocol).unwrap_or_default(),
                serde_json::to_string(&url.hostname).unwrap_or_default(),
                url.port,
                serde_json::to_string(&url.pathname).unwrap_or_default(),
                serde_json::to_string(&url.search).unwrap_or_default(),
                serde_json::to_string(&url.hash).unwrap_or_default(),
                serde_json::to_string(&url.host).unwrap_or_default(),
                serde_json::to_string(&url.origin).unwrap_or_default(),
            );
            context.eval(boa_engine::Source::from_bytes(&code))
        }
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(e)
        )),
    }
}

fn url_resolve_fn(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let base = args.first().and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
    let relative = args.get(1).and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
    match Url::resolve(&base, &relative) {
        Ok(resolved) => Ok(JsValue::from(JsString::from(resolved))),
        Err(e) => Err(boa_engine::JsError::from_native(boa_engine::JsNativeError::error().with_message(e))),
    }
}

fn url_encode_fn(_this: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let text = args.first().and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
    let encoded: String = text.bytes().map(|b| format!("%{:02X}", b)).collect();
    Ok(JsValue::from(JsString::from(encoded)))
}

fn url_decode_fn(_this: &JsValue, args: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let text = args.first().and_then(|v| v.as_string()).map(|s| s.to_std_string_escaped()).unwrap_or_default();
    let mut decoded = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    decoded.push(byte as char);
                    continue;
                }
            }
            decoded.push('%');
            decoded.push_str(&hex);
        } else {
            decoded.push(c);
        }
    }
    Ok(JsValue::from(JsString::from(decoded)))
}

pub struct UrlUtils;

impl UrlUtils {
    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(NativeFunction::from_fn_ptr(url_parse_fn), js_string!("parse"), 1usize)
            .function(NativeFunction::from_fn_ptr(url_resolve_fn), js_string!("resolve"), 2usize)
            .function(NativeFunction::from_fn_ptr(url_encode_fn), js_string!("encode"), 1usize)
            .function(NativeFunction::from_fn_ptr(url_decode_fn), js_string!("decode"), 1usize);
        let obj = builder.build();

        context.register_global_property(
            js_string!("URL"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        Ok(())
    }
}
