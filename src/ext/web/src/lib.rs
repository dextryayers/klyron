use deno_core::{extension, op2, Extension};
use deno_error::JsErrorBox;

extension!(
  klyron_web,
  ops = [
    op_web_fetch_ex,
    op_web_text_encode,
    op_web_text_decode,
    op_web_base64_encode,
    op_web_base64_decode,
  ],
  esm_entry_point = "ext:klyron_web/web.js",
  esm = [dir "js", "web.js"],
);

pub fn init() -> Extension {
  klyron_web::init()
}

#[derive(serde::Serialize)]
struct FetchResult {
  status: u16,
  status_text: String,
  headers: Vec<(String, String)>,
  body: String, // base64-encoded bytes
}

/// Perform an HTTP request synchronously (blocking reqwest) and return the
/// status, status text, response headers (as `[name, value]` pairs) and the
/// response body encoded as base64 (binary-safe).
#[op2]
#[string]
fn op_web_fetch_ex(
  #[string] url: String,
  #[string] method: String,
  #[string] headers_json: String,
  #[serde] body: Vec<u8>,
) -> Result<String, JsErrorBox> {
  let method = method.to_uppercase();
  let client = reqwest::blocking::Client::new();

  let mut builder = match method.as_str() {
    "GET" => client.get(&url),
    "POST" => client.post(&url),
    "PUT" => client.put(&url),
    "DELETE" => client.delete(&url),
    "HEAD" => client.head(&url),
    "PATCH" => client.patch(&url),
    "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url),
    _ => {
      let m = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|e| JsErrorBox::generic(format!("bad method: {e}")))?;
      client.request(m, &url)
    }
  };

  // Parse headers JSON (object or array of pairs).
  if !headers_json.is_empty() {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&headers_json) {
      match v {
        serde_json::Value::Object(map) => {
          for (k, val) in map {
            if let Some(s) = val.as_str() {
              builder = builder.header(&k, s);
            }
          }
        }
        serde_json::Value::Array(arr) => {
          for pair in arr {
            if let Some(p) = pair.as_array() {
              if p.len() == 2 {
                if let (Some(k), Some(v)) = (p[0].as_str(), p[1].as_str()) {
                  builder = builder.header(k, v);
                }
              }
            }
          }
        }
        _ => {}
      }
    }
  }

  if !body.is_empty() {
    builder = builder.body(body);
  }

  let resp = builder.send().map_err(|e| JsErrorBox::generic(format!("fetch {url}: {e}")))?;

  let status = resp.status().as_u16();
  let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
  let mut headers: Vec<(String, String)> = Vec::new();
  for (k, v) in resp.headers().iter() {
    headers.push((k.as_str().to_string(), v.to_str().unwrap_or("").to_string()));
  }

  let bytes = resp.bytes().map_err(|e| JsErrorBox::generic(format!("fetch body: {e}")))?;
  use base64::Engine;
  let body_b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

  let result = FetchResult {
    status,
    status_text: status_text,
    headers,
    body: body_b64,
  };
  serde_json::to_string(&result).map_err(|e| JsErrorBox::generic(format!("serialize: {e}")))
}

#[op2]
#[serde]
fn op_web_text_encode(#[string] text: String) -> Vec<u8> {
  text.into_bytes()
}

#[op2]
#[string]
fn op_web_text_decode(#[serde] bytes: Vec<u8>) -> String {
  String::from_utf8_lossy(&bytes).to_string()
}

#[op2]
#[string]
fn op_web_base64_encode(#[serde] data: Vec<u8>) -> String {
  use base64::Engine;
  base64::engine::general_purpose::STANDARD.encode(&data)
}

#[op2]
#[serde]
fn op_web_base64_decode(#[string] encoded: String) -> Result<Vec<u8>, JsErrorBox> {
  use base64::Engine;
  base64::engine::general_purpose::STANDARD.decode(&encoded).map_err(|e| JsErrorBox::generic(format!("base64 decode: {e}")))
}


#[cfg(test)]
mod integration_tests {
  use deno_core::{v8, FastString, JsRuntime, ModuleLoadOptions, ModuleLoadReferrer,
                  ModuleLoadResponse, ModuleLoader, ModuleSpecifier, RuntimeOptions};
  use std::rc::Rc;

  struct TestLoader;
  impl ModuleLoader for TestLoader {
    fn resolve(
      &self,
      specifier: &str,
      _referrer: &str,
      _kind: deno_core::ResolutionKind,
    ) -> deno_core::ModuleResolveResponse {
      Ok(ModuleSpecifier::parse(specifier).unwrap())
    }
    fn load(
      &self,
      _specifier: &ModuleSpecifier,
      _maybe_referrer: Option<&ModuleLoadReferrer>,
      _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
      ModuleLoadResponse::Sync(Err(deno_error::JsErrorBox::generic("unexpected load")))
    }
  }

  async fn run_js(source: &str) -> String {
    let mut runtime = JsRuntime::new(RuntimeOptions {
      extensions: vec![crate::init()],
      module_loader: Some(Rc::new(TestLoader)),
      ..Default::default()
    });
    let spec = ModuleSpecifier::parse("ext:klyron_test/main.mjs").unwrap();
    let id = runtime
      .load_main_es_module_from_code(&spec, source.to_string())
      .await
      .unwrap();
    runtime.mod_evaluate(id).await.unwrap();
    runtime
      .run_event_loop(deno_core::PollEventLoopOptions::default())
      .await
      .unwrap();
    let global = runtime
      .execute_script("read", FastString::from("globalThis.__RESULT__".to_string()))
      .unwrap();
    deno_core::scope!(scope, &mut runtime);
    let local = v8::Local::new(scope, global);
    match deno_core::serde_v8::from_v8::<Option<String>>(scope, local) {
      Ok(Some(s)) => s,
      _ => String::new(),
    }
  }

  // fetch() performs a real network request; run outside CI/tokio runtime.
  #[tokio::test]
  #[ignore = "performs a real network request"]
  async fn test_fetch_real() {
    let out = run_js(r#"
      const res = await fetch('https://example.com');
      const text = await res.text();
      globalThis.__RESULT__ = (res.status === 200 && typeof text === 'string') ? 'OK' : 'FAIL';
    "#).await;
    assert_eq!(out, "OK");
  }

  #[tokio::test]
  async fn test_web_globals_and_classes() {
    let out = run_js(r#"
      const ok =
        typeof globalThis.fetch === 'function' &&
        typeof globalThis.Request === 'function' &&
        typeof globalThis.Response === 'function' &&
        typeof globalThis.Headers === 'function' &&
        typeof globalThis.FormData === 'function' &&
        (new Headers({'X-Test': '1'}).get('x-test') === '1') &&
        (new Request('https://x.test', { method: 'POST' }).method === 'POST') &&
        (new Response('hi').status === 200);
      globalThis.__RESULT__ = ok ? 'OK' : 'FAIL';
    "#).await;
    assert_eq!(out, "OK");
  }
}
