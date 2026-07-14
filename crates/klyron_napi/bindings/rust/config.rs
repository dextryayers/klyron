use crate::types::NapiLoaderConfig;

#[derive(Debug, Clone)]
pub struct NapiConfig {
    pub loader: NapiLoaderConfig,
    pub napi_version: u32,
}

impl Default for NapiConfig {
    fn default() -> Self {
        Self {
            loader: NapiLoaderConfig::default(),
            napi_version: 9,
        }
    }
}

impl NapiConfig {
    pub fn new() -> Self { Self::default() }
    pub fn with_loader(mut self, loader: NapiLoaderConfig) -> Self {
        self.loader = loader;
        self
    }
}
