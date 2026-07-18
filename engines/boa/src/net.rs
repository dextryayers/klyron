use boa_engine::{Context, JsValue, NativeFunction, JsResult, js_string, JsString};
use boa_engine::property::Attribute;
use std::time::Duration;

fn net_fetch(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let url = args.first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();

    let result = fetch_url(&url, Duration::from_secs(10));
    match result {
        Ok(response) => {
            let status = response.status;
            let body = serde_json::to_string(&response.body).unwrap_or_else(|_| "\"\"".to_string());
            let code = format!("({{ status: {}, ok: {}, body: {} }})",
                status,
                if status >= 200 && status < 300 { "true" } else { "false" },
                body,
            );
            context.eval(boa_engine::Source::from_bytes(&code))
        }
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("fetch failed: {}", e))
        )),
    }
}

struct FetchResponse {
    status: u16,
    body: String,
}

fn fetch_url(url: &str, _timeout: Duration) -> Result<FetchResponse, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("unsupported protocol in URL: {}", url));
    }
    let output = std::process::Command::new("curl")
        .arg("-s")
        .arg("-w")
        .arg("%{http_code}")
        .arg("-o")
        .arg("-")
        .arg(url)
        .output()
        .map_err(|e| format!("curl execution failed: {}", e))?;

    if output.status.success() || output.status.code().map_or(false, |c| c > 0) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.len() >= 3 {
            let body_end = stdout.len() - 3;
            let body = stdout[..body_end].to_string();
            let status_str = &stdout[body_end..];
            let status: u16 = status_str.parse().unwrap_or(0);
            return Ok(FetchResponse { status, body });
        }
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!("curl error ({}): {}", output.status, stderr))
}

fn net_request(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let method = args.first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped().to_uppercase())
        .unwrap_or_else(|| "GET".to_string());
    let url = args.get(1)
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();
    let body = args.get(2)
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .unwrap_or_default();

    let output = std::process::Command::new("curl")
        .arg("-s")
        .arg("-X")
        .arg(&method)
        .arg("-d")
        .arg(&body)
        .arg("-w")
        .arg("%{http_code}")
        .arg("-o")
        .arg("-")
        .arg(&url)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.len() >= 3 {
                let body_end = stdout.len() - 3;
                let resp_body = &stdout[..body_end];
                let status_str = &stdout[body_end..];
                let status: u16 = status_str.parse().unwrap_or(0);
                let code = format!(
                    "({{ status: {}, body: {} }})",
                    status,
                    serde_json::to_string(resp_body).unwrap_or_else(|_| "\"\"".to_string()),
                );
                return context.eval(boa_engine::Source::from_bytes(&code));
            }
            Ok(JsValue::from(JsString::from(stdout.as_ref())))
        }
        Err(e) => Err(boa_engine::JsError::from_native(
            boa_engine::JsNativeError::error().with_message(format!("request failed: {}", e))
        )),
    }
}

pub struct Net;

impl Net {
    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(NativeFunction::from_fn_ptr(net_fetch), js_string!("fetch"), 1usize)
            .function(NativeFunction::from_fn_ptr(net_request), js_string!("request"), 3usize);
        let net_obj = builder.build();

        context.register_global_property(
            js_string!("net"),
            net_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;

        let fetch_fn = NativeFunction::from_fn_ptr(net_fetch);
        context.register_global_builtin_callable(js_string!("fetch"), 1usize, fetch_fn)
            .map_err(|e| crate::BoaError::from_js_error(&e))?;

        Ok(())
    }
}
