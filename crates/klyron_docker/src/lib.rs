pub mod compose;
pub mod container;
pub mod image;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use http::Uri;
use hyper::client::conn::http1::Builder;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::net::UnixStream;

pub use compose::{ComposeConfig, DockerManager};
pub use container::*;
pub use image::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockerProfile {
    Dev,
    Prod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerService {
    pub name: String,
    pub image: String,
    pub port: u16,
    pub env_vars: HashMap<String, String>,
    pub depends_on: Vec<String>,
    pub volumes: Vec<String>,
    pub health_check: Option<HealthCheckConfig>,
    pub profile: DockerProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub test: Vec<String>,
    pub interval: String,
    pub timeout: String,
    pub retries: u32,
    pub start_period: String,
}

#[derive(Debug, Clone)]
pub struct DockerConfig {
    pub image_name: String,
    pub port: u16,
    pub project_dir: PathBuf,
    pub profile: DockerProfile,
    pub additional_services: Vec<DockerService>,
    pub build_args: HashMap<String, String>,
    pub optimize: bool,
    pub use_build_cache: bool,
}

impl Default for DockerConfig {
    fn default() -> Self {
        DockerConfig {
            image_name: "klyron-app".into(),
            port: 3000,
            project_dir: PathBuf::from("."),
            profile: DockerProfile::Dev,
            additional_services: Vec::new(),
            build_args: HashMap::new(),
            optimize: true,
            use_build_cache: true,
        }
    }
}

const DOCKER_SOCKET: &str = "/var/run/docker.sock";

pub struct DockerClient {
    socket_path: String,
    inner: std::sync::Mutex<DockerClientInner>,
}

struct DockerClientInner {
    last_status: u16,
}

impl DockerClient {
    pub fn new() -> Self {
        Self {
            socket_path: DOCKER_SOCKET.to_string(),
            inner: std::sync::Mutex::new(DockerClientInner { last_status: 0 }),
        }
    }

    pub fn with_socket_path(path: &str) -> Self {
        Self {
            socket_path: path.to_string(),
            inner: std::sync::Mutex::new(DockerClientInner { last_status: 0 }),
        }
    }

    pub fn last_status(&self) -> u16 {
        self.inner.lock().unwrap().last_status
    }

    async fn send_request(
        &self,
        method: &str,
        endpoint: &str,
        body: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to Docker socket")?;
        let io = TokioIo::new(stream);

        let (mut sender, conn) = Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await
            .context("Failed to handshake with Docker daemon")?;

        tokio::spawn(async move {
            conn.await.ok();
        });

        let uri: Uri = endpoint.parse().context("Invalid endpoint URI")?;
        let body: http_body_util::Full<hyper::body::Bytes> = http_body_util::Full::new(body.to_vec().into());

        let req = http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Host", "localhost")
            .header("Content-Type", "application/json")
            .body(body)
            .context("Failed to build request")?;

        let resp = sender.send_request(req).await.context("Request failed")?;
        if let Ok(mut inner) = self.inner.lock() {
            inner.last_status = resp.status().as_u16();
        }

        let body_bytes = http_body_util::BodyExt::collect(resp.into_body())
            .await
            .context("Failed to read response body")?
            .to_bytes()
            .to_vec();

        Ok(body_bytes)
    }

    pub async fn get(&self, endpoint: &str) -> anyhow::Result<Vec<u8>> {
        self.send_request("GET", endpoint, &[]).await
    }

    pub async fn post(&self, endpoint: &str, body: &[u8]) -> anyhow::Result<Vec<u8>> {
        self.send_request("POST", endpoint, body).await
    }

    pub async fn delete(&self, endpoint: &str) -> anyhow::Result<Vec<u8>> {
        self.send_request("DELETE", endpoint, &[]).await
    }

    pub async fn ping(&self) -> anyhow::Result<String> {
        let resp = self.get("/_ping").await?;
        Ok(String::from_utf8_lossy(&resp).to_string())
    }

    pub async fn version(&self) -> anyhow::Result<serde_json::Value> {
        let resp = self.get("/version").await?;
        Ok(serde_json::from_slice(&resp).context("Failed to parse version")?)
    }

    pub async fn info(&self) -> anyhow::Result<serde_json::Value> {
        let resp = self.get("/info").await?;
        Ok(serde_json::from_slice(&resp).context("Failed to parse info")?)
    }
}

impl Default for DockerClient {
    fn default() -> Self {
        Self::new()
    }
}
