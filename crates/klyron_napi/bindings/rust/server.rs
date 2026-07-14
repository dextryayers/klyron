use crate::NapiLoader;
use std::sync::{Arc, Mutex};

pub struct NapiServer {
    loader: Arc<Mutex<NapiLoader>>,
}

impl NapiServer {
    pub fn new() -> Self {
        Self { loader: Arc::new(Mutex::new(NapiLoader::new())) }
    }

    pub fn load(&self, name: &str) -> anyhow::Result<()> {
        self.loader.lock().unwrap().load(name)?;
        Ok(())
    }

    pub fn list(&self) -> Vec<String> {
        self.loader.lock().unwrap().list_loaded()
    }

    pub fn unload(&self, name: &str) -> bool {
        self.loader.lock().unwrap().unload(name)
    }

    pub fn clear(&self) {
        self.loader.lock().unwrap().clear();
    }

    pub fn loader_ref(&self) -> Arc<Mutex<NapiLoader>> {
        self.loader.clone()
    }
}
