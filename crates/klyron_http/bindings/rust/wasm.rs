use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn klyron_http_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
