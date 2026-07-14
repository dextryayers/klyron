use std::time::Duration;

pub struct WebConfig {
    pub timeout: Duration,
    pub user_agent: String,
    pub max_redirects: u32,
}
impl Default for WebConfig {
    fn default() -> Self { Self { timeout: Duration::from_secs(60), user_agent: format!("KlyronWeb/{}", env!("CARGO_PKG_VERSION")), max_redirects: 10 } }
}
impl WebConfig { pub fn new() -> Self { Self::default() } }
