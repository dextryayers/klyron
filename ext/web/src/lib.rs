use deno_core::{extension, op2, Extension, JsErrorBox};

extension!(
  klyron_web,
  ops = [op_web_fetch, op_web_text_encode, op_web_text_decode, op_web_base64_encode, op_web_base64_decode],
  esm_entry_point = "ext:klyron_web/web.js",
  esm = [dir "js", "web.js"],
);

pub fn init() -> Extension {
  klyron_web::init()
}

#[op2]
#[string]
fn op_web_fetch(#[string] url: String) -> Result<String, JsErrorBox> {
  match reqwest::blocking::get(&url) {
    Ok(resp) => match resp.text() {
      Ok(text) => Ok(text),
      Err(e) => Err(JsErrorBox::generic(format!("fetch body: {e}"))),
    },
    Err(e) => Err(JsErrorBox::generic(format!("fetch {url}: {e}"))),
  }
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
