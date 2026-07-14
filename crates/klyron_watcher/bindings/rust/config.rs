#[derive(Debug, Clone)]
pub struct WatcherConfig {
    pub enabled: bool,
    pub verbose: bool,
}

impl Default for WatcherConfig {
    fn default() -> Self { Self { enabled: true, verbose: false } }
}
