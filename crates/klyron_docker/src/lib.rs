/// Docker integration for Klyron — generate Dockerfiles, docker-compose, build and run containers.
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Action to perform with Docker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockerAction {
    Init,
    Build,
    Run,
    Compose,
}

/// Docker configuration.
#[derive(Debug, Clone)]
pub struct DockerConfig {
    pub image_name: String,
    pub port: u16,
    pub project_dir: PathBuf,
}

/// Docker management interface.
pub struct DockerManager;

impl DockerManager {
    /// Create a new `DockerManager`.
    pub fn new() -> Self {
        Self
    }

    /// Generate a project-specific `Dockerfile` in `dir`.
    pub fn generate_dockerfile(dir: &Path) -> Result<()> {
        let project_type = Self::detect_project_type(dir);
        let dockerfile = match project_type.as_str() {
            "node" => {
                r#"FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY package*.json ./
EXPOSE 3000
CMD ["node", "dist/index.js"]
"#
            }
            "python" => {
                r#"FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
EXPOSE 8000
CMD ["python", "main.py"]
"#
            }
            "rust" => {
                r#"FROM rust:1.78 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/app /usr/local/bin/app
EXPOSE 8080
CMD ["app"]
"#
            }
            "go" => {
                r#"FROM golang:1.22 AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /app/server .

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder /app/server /server
EXPOSE 8080
CMD ["/server"]
"#
            }
            "php" => {
                r#"FROM php:8.2-cli
WORKDIR /app
COPY . .
EXPOSE 8000
CMD ["php", "-S", "0.0.0.0:8000", "-t", "public"]
"#
            }
            _ => {
                r#"FROM alpine:latest
WORKDIR /app
COPY . .
CMD ["sh"]
"#
            }
        };
        std::fs::write(dir.join("Dockerfile"), dockerfile)
            .context("Failed to write Dockerfile")
    }

    /// Generate a `docker-compose.yml` file in `dir`.
    pub fn generate_compose(dir: &Path) -> Result<()> {
        let project_type = Self::detect_project_type(dir);
        let port = match project_type.as_str() {
            "node" => "3000",
            "next" | "react" => "3000",
            "python" => "8000",
            "rust" => "8080",
            "go" => "8080",
            "php" => "8000",
            _ => "3000",
        };
        let name = dir.file_name().unwrap_or_default().to_string_lossy();

        let compose = format!(
            r#"version: "3.9"

services:
  {name}:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "{port}:{port}"
    volumes:
      - .:/app
      - /app/node_modules
    environment:
      - NODE_ENV=production
    restart: unless-stopped
"#
        );
        std::fs::write(dir.join("docker-compose.yml"), compose)
            .context("Failed to write docker-compose.yml")
    }

    /// Build a Docker image using the configuration.
    pub fn build(config: DockerConfig) -> Result<()> {
        let status = std::process::Command::new("docker")
            .args([
                "build",
                "-t",
                &config.image_name,
                ".",
            ])
            .current_dir(&config.project_dir)
            .status()
            .context("Failed to execute docker build")?;
        if !status.success() {
            anyhow::bail!("Docker build failed with exit code: {:?}", status.code());
        }
        Ok(())
    }

    /// Run a Docker container from the configuration.
    pub fn run(config: DockerConfig) -> Result<()> {
        let port_str = format!("{}:{}", config.port, config.port);
        let status = std::process::Command::new("docker")
            .args([
                "run",
                "-d",
                "--rm",
                "-p",
                &port_str,
                &config.image_name,
            ])
            .current_dir(&config.project_dir)
            .status()
            .context("Failed to execute docker run")?;
        if !status.success() {
            anyhow::bail!("Docker run failed with exit code: {:?}", status.code());
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn detect_project_type(dir: &Path) -> String {
        if dir.join("package.json").exists() {
            // Check for specific frameworks
            if let Ok(content) = std::fs::read_to_string(dir.join("package.json")) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
                        if deps.contains_key("next") {
                            return "next".into();
                        }
                    }
                }
            }
            return "node".into();
        }
        if dir.join("Cargo.toml").exists() {
            return "rust".into();
        }
        if dir.join("go.mod").exists() {
            return "go".into();
        }
        if dir.join("requirements.txt").exists() || dir.join("setup.py").exists() {
            return "python".into();
        }
        if dir.join("composer.json").exists() {
            return "php".into();
        }
        if dir.join("index.html").exists() || dir.join("public/index.html").exists() {
            return "static".into();
        }
        "node".into()
    }
}

impl Default for DockerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("klyron_docker_test_{}_{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_generate_dockerfile_node() {
        let dir = temp_dir();
        fs::write(dir.join("package.json"), r#"{"name":"test"}"#).unwrap();
        DockerManager::generate_dockerfile(&dir).expect("Generate failed");
        let content = fs::read_to_string(dir.join("Dockerfile")).unwrap();
        assert!(content.contains("FROM node"));
        assert!(content.contains("EXPOSE 3000"));
    }

    #[test]
    fn test_generate_dockerfile_rust() {
        let dir = temp_dir();
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
        DockerManager::generate_dockerfile(&dir).expect("Generate failed");
        let content = fs::read_to_string(dir.join("Dockerfile")).unwrap();
        assert!(content.contains("FROM rust"));
        assert!(content.contains("EXPOSE 8080"));
    }

    #[test]
    fn test_generate_dockerfile_go() {
        let dir = temp_dir();
        fs::write(dir.join("go.mod"), "module test\ngo 1.22").unwrap();
        DockerManager::generate_dockerfile(&dir).expect("Generate failed");
        let content = fs::read_to_string(dir.join("Dockerfile")).unwrap();
        assert!(content.contains("FROM golang"));
        assert!(content.contains("EXPOSE 8080"));
    }

    #[test]
    fn test_generate_compose() {
        let dir = temp_dir();
        fs::write(dir.join("package.json"), r#"{"name":"test"}"#).unwrap();
        DockerManager::generate_compose(&dir).expect("Generate failed");
        let content = fs::read_to_string(dir.join("docker-compose.yml")).unwrap();
        assert!(content.contains("services:"));
        assert!(content.contains("3000:3000"));
    }

    #[test]
    fn test_generate_dockerfile_python() {
        let dir = temp_dir();
        fs::write(dir.join("requirements.txt"), "fastapi\n").unwrap();
        DockerManager::generate_dockerfile(&dir).expect("Generate failed");
        let content = fs::read_to_string(dir.join("Dockerfile")).unwrap();
        assert!(content.contains("FROM python"));
        assert!(content.contains("EXPOSE 8000"));
    }

    #[test]
    fn test_default() {
        let mgr = DockerManager::default();
        // Just verify it compiles and creates fine
        let _ = mgr;
    }
}
