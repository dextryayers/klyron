use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum HttpScheme { Http, Https }

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
}
impl Default for ServerConfig {
    fn default() -> Self { Self { host: "0.0.0.0".into(), port: 3000, scheme: HttpScheme::Http, tls: None, cors_enabled: true, max_body_size: 10 * 1024 * 1024 } }
}

#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub method: String,
    pub path: String,
}

pub struct HttpServer {
    pub addr: SocketAddr,
    pub config: ServerConfig,
    pub routes: Vec<RouteConfig>,
}
impl HttpServer {
    pub fn new(host: &str, port: u16) -> Self { let addr: SocketAddr = format!("{host}:{port}").parse().unwrap_or(([0,0,0,0], port).into()); let config = ServerConfig { host: host.into(), port, ..Default::default() }; Self { addr, config, routes: Vec::new() } }
    pub fn with_config(config: ServerConfig) -> Self { let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse().unwrap_or(([0,0,0,0], config.port).into()); Self { addr, config, routes: Vec::new() } }
    pub fn route(mut self, method: &str, path: &str) -> Self { self.routes.push(RouteConfig { method: method.to_uppercase(), path: path.into() }); self }
    pub fn ws_route(mut self, path: &str) -> Self { self.routes.push(RouteConfig { method: "WS".into(), path: path.into() }); self }
    pub fn get_routes(&self) -> &[RouteConfig] { &self.routes }
    pub fn addr(&self) -> SocketAddr { self.addr }
    pub fn config(&self) -> &ServerConfig { &self.config }
}
