use axum::{routing::get, Json, Router};
use serde::Serialize;
use tower_http::cors::CorsLayer;

#[derive(Serialize)]
struct Health {
    status: String,
    service: String,
    version: String,
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok".to_string(),
        service: "{{ name }}".to_string(),
        version: "{{ version }}".to_string(),
    })
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let app = Router::new()
        .route("/api/health", get(health))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
