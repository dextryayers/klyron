/// Deployment abstraction for Klyron — generate config files and deploy to cloud platforms.
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// Supported deployment platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployPlatform {
    Vercel,
    Cloudflare,
    Railway,
    Fly,
    Docker,
}

/// Configuration for a deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub platform: DeployPlatform,
    pub preview: bool,
    pub project_dir: PathBuf,
}

/// Deployment orchestrator.
pub struct Deployment;

impl Deployment {
    /// Create a new `Deployment`.
    pub fn new() -> Self {
        Self
    }

    /// Deploy the project using the given configuration.
    ///
    /// This runs the appropriate CLI command for the selected platform.
    pub fn deploy(config: DeployConfig) -> Result<()> {
        // Ensure configs are generated
        Self::generate_config(&config.project_dir, config.platform)?;

        match config.platform {
            DeployPlatform::Vercel => {
                let mut cmd = std::process::Command::new("npx");
                cmd.args(["vercel", "--prod"])
                    .arg("--yes")
                    .current_dir(&config.project_dir);
                if config.preview {
                    cmd.arg("--prebuilt");
                }
                let status = cmd.status().context("Failed to run Vercel CLI")?;
                if !status.success() {
                    bail!("Vercel deploy failed with exit code: {:?}", status.code());
                }
            }
            DeployPlatform::Cloudflare => {
                let mut cmd = std::process::Command::new("npx");
                cmd.args(["wrangler", "deploy"])
                    .current_dir(&config.project_dir);
                if config.preview {
                    cmd.arg("--preview");
                }
                let status = cmd.status().context("Failed to run Wrangler CLI")?;
                if !status.success() {
                    bail!("Cloudflare deploy failed with exit code: {:?}", status.code());
                }
            }
            DeployPlatform::Railway => {
                let mut cmd = std::process::Command::new("railway");
                cmd.arg("up").current_dir(&config.project_dir);
                if config.preview {
                    cmd.args(["--environment", "preview"]);
                }
                let status = cmd.status().context("Failed to run Railway CLI")?;
                if !status.success() {
                    bail!("Railway deploy failed with exit code: {:?}", status.code());
                }
            }
            DeployPlatform::Fly => {
                let status = std::process::Command::new("flyctl")
                    .args(["deploy", "--remote-only"])
                    .current_dir(&config.project_dir)
                    .status()
                    .context("Failed to run Fly CLI")?;
                if !status.success() {
                    bail!("Fly deploy failed with exit code: {:?}", status.code());
                }
            }
            DeployPlatform::Docker => {
                let image_name = format!(
                    "klyron-deploy-{}",
                    config
                        .project_dir
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                );
                // Build
                let build_status = std::process::Command::new("docker")
                    .args(["build", "-t", &image_name, "."])
                    .current_dir(&config.project_dir)
                    .status()
                    .context("Failed to run Docker build")?;
                if !build_status.success() {
                    bail!("Docker build failed with exit code: {:?}", build_status.code());
                }
                // Run (detached)
                let run_status = std::process::Command::new("docker")
                    .args(["run", "-d", "--rm", "-p", "3000:3000", &image_name])
                    .current_dir(&config.project_dir)
                    .status()
                    .context("Failed to run Docker container")?;
                if !run_status.success() {
                    bail!("Docker run failed with exit code: {:?}", run_status.code());
                }
            }
        }

        Ok(())
    }

    /// Generate platform-specific configuration files in `dir`.
    pub fn generate_config(dir: &Path, platform: DeployPlatform) -> Result<()> {
        match platform {
            DeployPlatform::Vercel => Self::generate_vercel_config(dir),
            DeployPlatform::Cloudflare => Self::generate_cloudflare_config(dir),
            DeployPlatform::Railway => Self::generate_railway_config(dir),
            DeployPlatform::Fly => Self::generate_fly_config(dir),
            DeployPlatform::Docker => Self::generate_docker_config(dir),
        }
    }

    /// Auto-detect deployment platform from existing configuration files in `dir`.
    pub fn detect_platform(dir: &Path) -> Option<DeployPlatform> {
        if dir.join("vercel.json").exists() || dir.join(".vercel").exists() {
            return Some(DeployPlatform::Vercel);
        }
        if dir.join("wrangler.toml").exists() {
            return Some(DeployPlatform::Cloudflare);
        }
        if dir.join("railway.json").exists() {
            return Some(DeployPlatform::Railway);
        }
        if dir.join("fly.toml").exists() {
            return Some(DeployPlatform::Fly);
        }
        if dir.join("Dockerfile").exists() || dir.join("docker-compose.yml").exists() {
            return Some(DeployPlatform::Docker);
        }
        None
    }

    // -----------------------------------------------------------------------
    // Config generators
    // -----------------------------------------------------------------------

    fn generate_vercel_config(dir: &Path) -> Result<()> {
        let project_type = Self::detect_project_type(dir);
        let config = match project_type.as_str() {
            "node" | "next" | "react" => {
                let (build_cmd, out_dir) = if project_type == "next" {
                    ("next build", ".next")
                } else {
                    ("npm run build", "dist")
                };
                serde_json::json!({
                    "framework": project_type,
                    "buildCommand": build_cmd,
                    "outputDirectory": out_dir,
                    "installCommand": "npm install",
                })
            }
            "python" => serde_json::json!({
                "buildCommand": "pip install -r requirements.txt",
                "outputDirectory": "public",
            }),
            "rust" => serde_json::json!({
                "buildCommand": "cargo build --release",
                "outputDirectory": "target/release",
            }),
            "go" => serde_json::json!({
                "buildCommand": "go build -o dist/app",
                "outputDirectory": "dist",
            }),
            "php" => serde_json::json!({
                "buildCommand": "composer install --no-dev",
                "outputDirectory": "public",
            }),
            _ => serde_json::json!({
                "version": 2,
                "builds": [{ "src": "**/*.js", "use": "@vercel/node" }],
            }),
        };
        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(dir.join("vercel.json"), content)
            .context("Failed to write vercel.json")
    }

    fn generate_cloudflare_config(dir: &Path) -> Result<()> {
        let project_type = Self::detect_project_type(dir);
        let name = dir.file_name().unwrap_or_default().to_string_lossy();
        let config = format!(
            r#"name = "{name}"
compatibility_date = "2024-12-01"
main = "src/index.js"
"#
        );
        // Add language-specific config
        let config = match project_type.as_str() {
            "rust" => config + "build.command = \"cargo build --release\"\n",
            "python" => config + "build.command = \"pip install -r requirements.txt\"\n",
            "go" => config + "build.command = \"go build -o dist/app\"\n",
            _ => config,
        };
        std::fs::write(dir.join("wrangler.toml"), config)
            .context("Failed to write wrangler.toml")
    }

    fn generate_railway_config(dir: &Path) -> Result<()> {
        let project_type = Self::detect_project_type(dir);
        let (builder, build_cmd, start_cmd) = match project_type.as_str() {
            "node" => ("NIXPACKS", Some("npm run build"), Some("npm start")),
            "python" => ("NIXPACKS", None, Some("python main.py")),
            "rust" => ("CARGO", Some("cargo build --release"), Some("./target/release/app")),
            "go" => ("GO", Some("go build -o app"), Some("./app")),
            "php" => ("NIXPACKS", Some("composer install --no-dev"), Some("php -S 0.0.0.0:8000 -t public")),
            _ => ("NIXPACKS", None, None),
        };
        let config = serde_json::json!({
            "build": {
                "builder": builder,
                "buildCommand": build_cmd,
            },
            "deploy": {
                "startCommand": start_cmd,
                "healthcheckPath": "/",
            },
        });
        let content = serde_json::to_string_pretty(&config)?;
        std::fs::write(dir.join("railway.json"), content)
            .context("Failed to write railway.json")
    }

    fn generate_fly_config(dir: &Path) -> Result<()> {
        let project_type = Self::detect_project_type(dir);
        let name = dir.file_name().unwrap_or_default().to_string_lossy();
        let (_build_cmd, _start_cmd, internal_port) = match project_type.as_str() {
            "node" => ("npm run build", "npm start", "3000"),
            "next" => ("next build", "next start", "3000"),
            "react" => ("npm run build", "npx serve -s dist -l 3000", "3000"),
            "python" => ("echo 'ok'", "python main.py", "8000"),
            "rust" => ("cargo build --release", "./target/release/app", "8080"),
            "go" => ("go build -o app", "./app", "8080"),
            "php" => ("composer install --no-dev", "php -S 0.0.0.0:8000 -t public", "8000"),
            _ => ("npm run build", "npm start", "3000"),
        };
        let config = format!(
            r#"app = "{name}"
primary_region = "iad"

[build]
  builder = "heroku/buildpacks:20"

[http_service]
  internal_port = {internal_port}
  force_https = true

[env]
  NODE_ENV = "production"
"#
        );
        std::fs::write(dir.join("fly.toml"), config).context("Failed to write fly.toml")
    }

    fn generate_docker_config(dir: &Path) -> Result<()> {
        let project_type = Self::detect_project_type(dir);
        let dockerfile = match project_type.as_str() {
            "node" | "next" | "react" => {
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

    fn detect_project_type(dir: &Path) -> String {
        if dir.join("package.json").exists() {
            if let Ok(content) = std::fs::read_to_string(dir.join("package.json")) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
                        if deps.contains_key("next") {
                            return "next".into();
                        }
                        if deps.contains_key("react") || deps.contains_key("react-dom") {
                            return "react".into();
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
        "node".into()
    }
}

impl Default for Deployment {
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
        let dir = std::env::temp_dir().join(format!("klyron_deploy_test_{}_{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_detect_platform_vercel() {
        let dir = temp_dir();
        fs::write(dir.join("vercel.json"), "{}").unwrap();
        assert_eq!(Deployment::detect_platform(&dir), Some(DeployPlatform::Vercel));
    }

    #[test]
    fn test_detect_platform_cloudflare() {
        let dir = temp_dir();
        fs::write(dir.join("wrangler.toml"), "").unwrap();
        assert_eq!(
            Deployment::detect_platform(&dir),
            Some(DeployPlatform::Cloudflare)
        );
    }

    #[test]
    fn test_detect_platform_none() {
        let dir = temp_dir();
        assert_eq!(Deployment::detect_platform(&dir), None);
    }

    #[test]
    fn test_generate_vercel_config() {
        let dir = temp_dir();
        fs::write(dir.join("package.json"), r#"{"name":"test"}"#).unwrap();
        Deployment::generate_config(&dir, DeployPlatform::Vercel).expect("Generate failed");
        assert!(dir.join("vercel.json").exists());
        let content = fs::read_to_string(dir.join("vercel.json")).unwrap();
        assert!(content.contains("framework") || content.contains("version"));
    }

    #[test]
    fn test_generate_cloudflare_config() {
        let dir = temp_dir();
        fs::write(dir.join("package.json"), r#"{"name":"test"}"#).unwrap();
        Deployment::generate_config(&dir, DeployPlatform::Cloudflare).expect("Generate failed");
        assert!(dir.join("wrangler.toml").exists());
    }

    #[test]
    fn test_generate_docker_config() {
        let dir = temp_dir();
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        Deployment::generate_config(&dir, DeployPlatform::Docker).expect("Generate failed");
        assert!(dir.join("Dockerfile").exists());
        let content = fs::read_to_string(dir.join("Dockerfile")).unwrap();
        assert!(content.contains("FROM rust"));
    }

    #[test]
    fn test_serialization() {
        let config = DeployConfig {
            platform: DeployPlatform::Vercel,
            preview: true,
            project_dir: PathBuf::from("/tmp/test"),
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: DeployConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.platform, DeployPlatform::Vercel);
        assert!(back.preview);
    }
}
