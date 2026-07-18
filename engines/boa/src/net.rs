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
            let ok = status >= 200 && status < 300;
            let body_str = JsString::from(response.body.as_str());

            let obj = boa_engine::object::ObjectInitializer::new(context)
                .property(js_string!("status"), status as f64, boa_engine::property::Attribute::all())
                .property(js_string!("ok"), ok, boa_engine::property::Attribute::all())
                .property(js_string!("body"), body_str, boa_engine::property::Attribute::all())
                .build();
            Ok(JsValue::from(obj))
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

fn fetch_url(url: &str, timeout: Duration) -> Result<FetchResponse, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("unsupported protocol in URL: {}", url));
    }
    let config = ureq::config::Config::builder()
        .timeout_connect(Some(timeout))
        .timeout_per_call(Some(timeout))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    let response = agent.get(url)
        .call()
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    let status = response.status().as_u16();
    let body = response.into_body().read_to_string()
        .map_err(|e| format!("failed to read response body: {}", e))?;
    Ok(FetchResponse { status, body })
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

    let result = (|| -> Result<FetchResponse, String> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(format!("unsupported protocol in URL: {}", url));
        }
        let config = ureq::config::Config::builder()
            .timeout_connect(Some(Duration::from_secs(30)))
            .timeout_per_call(Some(Duration::from_secs(30)))
            .build();
        let agent = ureq::Agent::new_with_config(config);
        let response = match method.as_str() {
            "GET" => agent.get(&url).call(),
            "POST" => agent.post(&url).send(body.as_str()),
            "PUT" => agent.put(&url).send(body.as_str()),
            "DELETE" => agent.delete(&url).call(),
            "PATCH" => agent.patch(&url).send(body.as_str()),
            "HEAD" => agent.head(&url).call(),
            _ => agent.get(&url).call(),
        };
        let response = response.map_err(|e| format!("HTTP request failed: {}", e))?;
        let status = response.status().as_u16();
        let resp_body = response.into_body().read_to_string()
            .map_err(|e| format!("failed to read response body: {}", e))?;
        Ok(FetchResponse { status, body: resp_body })
    })();

    match result {
        Ok(response) => {
            let body_str = JsString::from(response.body.as_str());
            let obj = boa_engine::object::ObjectInitializer::new(context)
                .property(js_string!("status"), response.status as f64, boa_engine::property::Attribute::all())
                .property(js_string!("body"), body_str, boa_engine::property::Attribute::all())
                .build();
            Ok(JsValue::from(obj))
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
