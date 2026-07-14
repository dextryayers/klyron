use crate::config::WatcherConfig;

pub struct WatcherBuilder {
    config: WatcherConfig,
}

impl WatcherBuilder {
    pub fn new() -> Self { Self { config: WatcherConfig::default() } }
    pub fn build(self) -> WatcherConfig { self.config }
}

impl Default for WatcherBuilder {
    fn default() -> Self { Self::new() }
}
