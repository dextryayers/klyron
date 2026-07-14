use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::Method;
use axum::response::Json;
use axum::routing::{any, delete, get, head, options, patch, post, put};
use axum::Router;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub method: String,
    pub path: String,
}

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
    #[inline]
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

#[derive(Clone)]
pub struct ConnectionPool {
    connections: Arc<AtomicUsize>,
    max: usize,
}

impl ConnectionPool {
    #[inline]
    pub fn new(max: usize) -> Self {
        Self { connections: Arc::new(AtomicUsize::new(0)), max }
    }

    #[inline]
    pub fn try_acquire(&self) -> bool {
        let current = self.connections.fetch_add(1, Ordering::SeqCst);
        if current < self.max {
            true
        } else {
            self.connections.fetch_sub(1, Ordering::SeqCst);
            false
        }
    }

    #[inline]
    pub fn release(&self) {
        self.connections.fetch_sub(1, Ordering::SeqCst);
    }

    #[inline]
    pub fn active(&self) -> usize {
        self.connections.load(Ordering::SeqCst)
    }
}

pub struct HttpServer {
    app: Router,
    addr: SocketAddr,
    config: ServerConfig,
    routes: Vec<RouteConfig>,
    pool: ConnectionPool,
}

impl HttpServer {
    pub fn new(host: &str, port: u16) -> Self {
        let addr: SocketAddr = format!("{host}:{port}").parse().unwrap_or_else(|_| {
            ([0, 0, 0, 0], port).into()
        });
        let app = Router::new()
            .route("/", get(|| async { "Klyron HTTP Server" }))
            .route("/health", get(|| async { Json(serde_json::json!({ "status": "ok", "version": "0.1.0" })) }))
            .route("/api/echo", post(|body: String| async move { Json(serde_json::json!({ "echo": body })) }))
            .layer(
                CorsLayer::new()
                    .allow_origin(tower_http::cors::Any)
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
                    .allow_headers(tower_http::cors::Any),
            );
        Self {
            app,
            addr,
            config: ServerConfig::default(),
            routes: vec![
                RouteConfig { method: "GET".to_string(), path: "/".to_string() },
                RouteConfig { method: "GET".to_string(), path: "/health".to_string() },
                RouteConfig { method: "POST".to_string(), path: "/api/echo".to_string() },
            ],
            pool: ConnectionPool::new(1024),
        }
    }

    pub fn with_config(config: ServerConfig) -> Self {
        let addr: SocketAddr = format!("{}:{}", config.host, config.port)
            .parse()
            .unwrap_or_else(|_| ([0, 0, 0, 0], config.port).into());
        let mut app = Router::new()
            .route("/", get(|| async { "Klyron HTTP Server" }))
            .route("/health", get(|| async { Json(serde_json::json!({ "status": "ok" })) }));

        if config.cors_enabled {
            app = app.layer(
                CorsLayer::new()
                    .allow_origin(tower_http::cors::Any)
                    .allow_methods(tower_http::cors::Any)
                    .allow_headers(tower_http::cors::Any),
            );
        }

        app = app.layer(TraceLayer::new_for_http());

        let pool = ConnectionPool::new(config.max_connections);

        Self { app, addr, config, routes: Vec::new(), pool }
    }

    #[inline]
    pub fn route(mut self, method: &str, path: &str) -> Self {
        let path = path.to_string();
        let app = match method.to_uppercase().as_str() {
            "GET" => self.app.route(&path, get(handler_empty)),
            "POST" => self.app.route(&path, post(handler_empty)),
            "PUT" => self.app.route(&path, put(handler_empty)),
            "DELETE" => self.app.route(&path, delete(handler_empty)),
            "PATCH" => self.app.route(&path, patch(handler_empty)),
            "HEAD" => self.app.route(&path, head(handler_empty)),
            "OPTIONS" => self.app.route(&path, options(handler_empty)),
            _ => self.app.route(&path, any(handler_empty)),
        };
        self.app = app;
        self.routes.push(RouteConfig {
            method: method.to_uppercase(),
            path,
        });
        self
    }

    #[inline]
    pub fn ws_route(mut self, path: &str) -> Self {
        let path = path.to_string();
        let state = Arc::new(());
        self.app = self.app.route(
            &path,
            get(move |ws: WebSocketUpgrade| ws_handler(ws, state.clone())),
        );
        self.routes.push(RouteConfig {
            method: "WS".to_string(),
            path,
        });
        self
    }

    #[inline]
    pub fn get_routes(&self) -> &[RouteConfig] {
        &self.routes
    }

    #[inline]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    #[inline]
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    #[inline]
    pub fn pool(&self) -> &ConnectionPool {
        &self.pool
    }

    pub async fn serve(self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        println!("Klyron HTTP server listening on http://{}", self.addr);

        if self.config.enable_http2 {
            self.serve_h2(listener).await
        } else {
            axum::serve(listener, self.app).await?;
            Ok(())
        }
    }

    async fn serve_h2(self, listener: tokio::net::TcpListener) -> anyhow::Result<()> {
        loop {
            let (stream, peer) = listener.accept().await?;
            if !self.pool.try_acquire() {
                eprintln!("Connection limit reached, dropping {peer}");
                continue;
            }
            let app = self.app.clone();
            let pool = self.pool.clone();

            tokio::spawn(async move {
                let io = hyper_util::rt::TokioIo::new(stream);
                let svc = hyper_util::service::TowerToHyperService::new(app.into_service());

                if let Err(e) = hyper::server::conn::http1::Builder::new()
                    .keep_alive(true)
                    .serve_connection(io, svc)
                    .await
                {
                    if !e.is_incomplete_message() {
                        eprintln!("Connection error from {peer}: {e}");
                    }
                }
                pool.release();
            });
        }
    }

    pub async fn serve_tls(self, cert_path: &str, key_path: &str) -> anyhow::Result<()> {
        use std::fs;

        let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(fs::File::open(cert_path)?))
            .collect::<Result<Vec<_>, _>>()?;
        let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(fs::File::open(key_path)?))?
            .ok_or_else(|| anyhow::anyhow!("No private key found in {key_path}"))?;

        let tls_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_config));
        let listener = tokio::net::TcpListener::bind(self.addr).await?;

        println!("Klyron HTTPS server listening on https://{}", self.addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let app = self.app.clone();
            let pool = self.pool.clone();

            tokio::spawn(async move {
                let tls_stream = match acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => { eprintln!("TLS accept error: {e}"); return; }
                };
                let io = hyper_util::rt::TokioIo::new(tls_stream);
                let svc = hyper_util::service::TowerToHyperService::new(app.into_service());
                let conn = hyper::server::conn::http1::Builder::new()
                    .keep_alive(true)
                    .serve_connection(io, svc);
                if let Err(e) = conn.await {
                    eprintln!("TLS connection error: {e}");
                }
                pool.release();
            });
        }
    }
}

async fn handler_empty() -> &'static str {
    "Klyron HTTP"
}

async fn ws_handler(ws: WebSocketUpgrade, state: Arc<()>) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, _state: Arc<()>) {
    loop {
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {
                if socket.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
            Some(Ok(Message::Ping(data))) => {
                if socket.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Some(Ok(Message::Close(_))) | None => break,
            _ => {}
        }
    }
}

#[derive(Clone)]
pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    #[inline]
    pub fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(32)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self { client })
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<String> {
        let resp = self.client.get(url).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn post(&self, url: &str, body: &str) -> anyhow::Result<String> {
        let resp = self.client.post(url).body(body.to_string()).send().await?;
        Ok(resp.text().await?)
    }

    pub async fn post_json<T: Serialize>(&self, url: &str, data: &T) -> anyhow::Result<String> {
        let resp = self.client.post(url).json(data).send().await?;
        Ok(resp.text().await?)
    }

    #[inline]
    pub fn inner(&self) -> &reqwest::Client {
        &self.client
    }
}

pub async fn serve_dir(host: &str, port: u16, dir: &str) -> anyhow::Result<()> {
    let app = Router::new()
        .fallback_service(ServeDir::new(dir))
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Serving directory '{}' at http://{}", dir, addr);
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn serve_static(host: &str, port: u16, dir: &str) -> anyhow::Result<()> {
    serve_dir(host, port, dir).await
}

pub async fn fetch_url(url: &str) -> anyhow::Result<String> {
    let resp = reqwest::get(url).await?;
    Ok(resp.text().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_new() {
        let _server = HttpServer::new("127.0.0.1", 0);
    }

    #[test]
    fn test_server_with_config() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            scheme: HttpScheme::Http,
            tls: None,
            cors_enabled: true,
            max_body_size: 5 * 1024 * 1024,
            max_connections: 512,
            keep_alive_timeout: Duration::from_secs(60),
            enable_http2: true,
        };
        let server = HttpServer::with_config(config);
        assert_eq!(server.addr().port(), 8080);
    }

    #[test]
    fn test_route_registration() {
        let server = HttpServer::new("127.0.0.1", 0)
            .route("GET", "/api/users")
            .route("POST", "/api/users")
            .route("DELETE", "/api/users/{id}");
        assert_eq!(server.get_routes().len(), 6);
    }

    #[test]
    fn test_ws_route() {
        let server = HttpServer::new("127.0.0.1", 0)
            .ws_route("/ws/chat");
        let routes = server.get_routes();
        assert!(routes.iter().any(|r| r.method == "WS" && r.path == "/ws/chat"));
    }

    #[test]
    fn test_connection_pool() {
        let pool = ConnectionPool::new(2);
        assert!(pool.try_acquire());
        assert!(pool.try_acquire());
        assert!(!pool.try_acquire());
        pool.release();
        assert!(pool.try_acquire());
    }
}
