use std::sync::Arc;
use std::time::Duration;
use axum::http::{HeaderMap, Request};
use axum::middleware::Next;
use axum::response::Response;
use tokio::sync::Semaphore;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }
}

pub async fn rate_limit_layer(
    state: Arc<RateLimiter>,
    req: Request<axum::body::Body>,
    next: Next<axum::body::Body>,
) -> Result<Response, axum::response::Response> {
    let _permit = state
        .semaphore
        .acquire()
        .await
        .map_err(|_| axum::response::Response::builder()
            .status(429)
            .body(axum::body::Body::from("Too Many Requests"))
            .unwrap())?;
    Ok(next.run(req).await)
}

#[derive(Clone)]
pub struct RequestLogger {
    counter: Arc<AtomicUsize>,
}

impl RequestLogger {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn next_id(&self) -> usize {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }
}

pub fn request_id_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

pub fn forwarded_for(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

pub fn user_agent(headers: &HeaderMap) -> &str {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
}

#[derive(Clone)]
pub struct TimeoutConfig {
    pub request_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(60),
        }
    }
}
