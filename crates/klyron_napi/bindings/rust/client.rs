use crate::{NapiLoader, NapiModule};

pub struct NapiClient {
    loader: NapiLoader,
}

impl NapiClient {
    pub fn new() -> Self {
        Self { loader: NapiLoader::new() }
    }

    pub fn load(&mut self, name: &str) -> anyhow::Result<&NapiModule> {
        self.loader.load(name)
    }

    pub fn list(&self) -> Vec<String> {
        self.loader.list_loaded()
    }

    pub fn unload(&mut self, name: &str) -> bool {
        self.loader.unload(name)
    }

    pub fn clear(&mut self) {
        self.loader.clear();
    }

    pub fn version(&self) -> u32 {
        self.loader.napi_version()
    }
}
