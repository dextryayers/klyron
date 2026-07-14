use std::net::SocketAddr;

use axum::http::Method;
use axum::response::Json;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub struct HttpServer {
    app: Router,
    addr: SocketAddr,
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
        Self { app, addr }
    }

    pub async fn serve(self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        println!("Klyron HTTP server listening on http://{}", self.addr);
        axum::serve(listener, self.app).await?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_server_new() {
        let _server = HttpServer::new("127.0.0.1", 0);
    }
}
