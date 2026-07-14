use crate::types::HttpServer;

pub struct ServerManager;
impl ServerManager {
    pub fn new() -> Self { Self }
    pub fn create(host: &str, port: u16) -> HttpServer { HttpServer::new(host, port) }
}
impl Default for ServerManager { fn default() -> Self { Self::new() } }
