use wasm_bindgen::prelude::*;
use crate::NapiLoader;

#[wasm_bindgen]
pub struct WasmNapiLoader {
    inner: NapiLoader,
}

#[wasm_bindgen]
impl WasmNapiLoader {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: NapiLoader::new() }
    }

    #[wasm_bindgen]
    pub fn is_napi_module(name: &str) -> bool {
        NapiLoader::is_napi_module(name)
    }

    #[wasm_bindgen]
    pub fn napi_version(&self) -> u32 {
        self.inner.napi_version()
    }

    #[wasm_bindgen]
    pub fn list_loaded(&self) -> Vec<JsValue> {
        self.inner.list_loaded().into_iter().map(JsValue::from).collect()
    }

    #[wasm_bindgen]
    pub fn is_loaded(&self, name: &str) -> bool {
        self.inner.is_loaded(name)
    }

    #[wasm_bindgen]
    pub fn unload(&mut self, name: &str) -> bool {
        self.inner.unload(name)
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
