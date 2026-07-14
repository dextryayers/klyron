pub struct WatcherClient {
    config: crate::config::WatcherConfig,
}

impl WatcherClient {
    pub fn new() -> Self { Self { config: crate::config::WatcherConfig::default() } }
    pub fn config(&self) -> &crate::config::WatcherConfig { &self.config }
    pub fn version(&self) -> &'static str { "1.0.0" }
}
