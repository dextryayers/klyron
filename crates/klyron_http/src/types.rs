use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum HttpScheme {
    Http,
    Https,
}

#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub scheme: HttpScheme,
    pub tls: Option<TlsConfig>,
    pub cors_enabled: bool,
    pub max_body_size: usize,
    pub max_connections: usize,
    pub keep_alive_timeout: Duration,
    pub enable_http2: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            scheme: HttpScheme::Http,
            tls: None,
            cors_enabled: true,
            max_body_size: 10 * 1024 * 1024,
            max_connections: 1024,
            keep_alive_timeout: Duration::from_secs(30),
            enable_http2: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub method: String,
    pub path: String,
}
