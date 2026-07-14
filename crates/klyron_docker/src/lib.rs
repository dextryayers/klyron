use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
    }
  }
}

pub struct DockerManager;

impl DockerManager {
  pub fn new() -> Self {
    Self
  }

  pub fn generate_dockerfile(dir: &Path) -> Result<()> {
    let config = DockerConfig::default();
    Self::generate_dockerfile_with_config(dir, &config)
  }

  pub fn generate_dockerfile_with_config(dir: &Path, config: &DockerConfig) -> Result<()> {
    let project_type = Self::detect_project_type(dir);
    let dockerfile = match project_type.as_str() {
      "node" | "next" | "react" => {
        let build_args: String = config.build_args.iter()
          .map(|(k, v)| format!("ARG {k}={v}\n"))
          .collect();
        format!(
          r#"# syntax=docker/dockerfile:1.4
{build_args}
FROM node:20-alpine AS builder
WORKDIR /app
RUN --mount=type=cache,target=/root/.npm \
    npm config set cache /root/.npm
COPY package*.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci
COPY . .
RUN npm run build

FROM node:20-alpine AS runner
WORKDIR /app
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 klyron
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY package*.json ./
RUN chown -R klyron:nodejs /app
USER klyron
EXPOSE {port}

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:{port}/ || exit 1

CMD ["node", "dist/index.js"]
"#,
          port = config.port,
          build_args = build_args,
        )
      }
      "rust" => {
        let build_args: String = config.build_args.iter()
          .map(|(k, v)| format!("ARG {k}={v}\n"))
          .collect();
        format!(
          r#"# syntax=docker/dockerfile:1.4
{build_args}
FROM rust:{rust_ver} AS builder
WORKDIR /app
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    mkdir src && echo "fn main() {{}}" > src/main.rs
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release 2>/dev/null || true
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/app /usr/local/bin/app
EXPOSE {port}

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/app", "--health-check"] || exit 1

CMD ["app"]
"#,
          rust_ver = "1.78",
          port = config.port,
          build_args = build_args,
        )
      }
      "python" => {
        let build_args: String = config.build_args.iter()
          .map(|(k, v)| format!("ARG {k}={v}\n"))
          .collect();
        format!(
          r#"# syntax=docker/dockerfile:1.4
{build_args}
FROM python:{py_ver}-slim AS builder
WORKDIR /app
RUN --mount=type=cache,target=/root/.cache/pip \
    pip install --no-cache-dir -r requirements.txt
COPY . .

FROM python:{py_ver}-slim
WORKDIR /app
COPY --from=builder /app /app
RUN adduser --system --uid 1001 klyron && chown -R klyron /app
USER klyron
EXPOSE {port}

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD python -c "import http.client; http.client.HTTPConnection('localhost', {port}).request('GET', '/')" || exit 1

CMD ["python", "main.py"]
"#,
          py_ver = "3.12",
          port = config.port,
          build_args = build_args,
        )
      }
      "go" => format!(
        r#"# syntax=docker/dockerfile:1.4
FROM golang:{go_ver} AS builder
WORKDIR /app
RUN --mount=type=cache,target=/go/pkg/mod
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /app/server .

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder /app/server /server
EXPOSE {port}

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:{port}/ || exit 1

CMD ["/server"]
"#,
        go_ver = "1.22",
        port = config.port,
      ),
      _ => r#"FROM alpine:latest
WORKDIR /app
COPY . .
CMD ["sh"]
"#.into(),
    };

    let dockerfile_path = dir.join("Dockerfile");
    std::fs::write(&dockerfile_path, dockerfile).context("Failed to write Dockerfile")
  }

  pub fn generate_dockerignore(dir: &Path) -> Result<()> {
    let content = r#"node_modules
.git
target
dist
.env
.env.local
*.log
.gitignore
Dockerfile
docker-compose.yml
.idea
.vscode
"#;
    std::fs::write(dir.join(".dockerignore"), content).context("Failed to write .dockerignore")
  }

  pub fn generate_compose(dir: &Path) -> Result<()> {
    let config = DockerConfig::default();
    Self::generate_compose_with_config(dir, &config)
  }

  pub fn generate_compose_with_config(dir: &Path, config: &DockerConfig) -> Result<()> {
    let name = dir.file_name().unwrap_or_default().to_string_lossy();

    let all_services = if config.additional_services.is_empty() {
      Self::default_infra_services()
    } else {
      config.additional_services.clone()
    };

    let mut compose = format!(
      r#"# docker-compose.yml generated by klyron_docker
# Profile: {profile:?}

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
      - NODE_ENV={env}
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:{port}/"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    profiles:
      - {profile:?}

"#,
      name = name,
      port = config.port,
      env = match config.profile { DockerProfile::Dev => "development", DockerProfile::Prod => "production" },
      profile = config.profile,
    );

    for svc in &all_services {
      let env_vars: String = svc.env_vars.iter()
        .map(|(k, v)| format!("      {k}: \"{v}\""))
        .collect::<Vec<_>>()
        .join("\n");

      let depends: String = if svc.depends_on.is_empty() {
        String::new()
      } else {
        format!("\n    depends_on:\n      - {}\n", svc.depends_on.join("\n      - "))
      };

      let health: String = if let Some(hc) = &svc.health_check {
        format!(
          r#"
    healthcheck:
      test: [{test}]
      interval: {interval}
      timeout: {timeout}
      retries: {retries}
      start_period: {start_period}
"#,
          test = hc.test.iter().map(|t| format!("\"{t}\"")).collect::<Vec<_>>().join(", "),
          interval = hc.interval,
          timeout = hc.timeout,
          retries = hc.retries,
          start_period = hc.start_period,
        )
      } else {
        String::new()
      };

      compose.push_str(&format!(
        r#"  {name}:
    image: {image}
    ports:
      - "{port}:{port}"
    environment:
{env_vars}{depends}{health}
    restart: unless-stopped
    profiles:
      - {profile:?}

"#,
        name = svc.name,
        image = svc.image,
        port = svc.port,
        env_vars = env_vars,
        depends = depends,
        health = health,
        profile = svc.profile,
      ));
    }

    let compose_path = dir.join("docker-compose.yml");
    std::fs::write(&compose_path, compose).context("Failed to write docker-compose.yml")
  }

  pub fn build(config: DockerConfig) -> Result<()> {
    let mut cmd = std::process::Command::new("docker");
    cmd.args(["build", "-t", &config.image_name, "."]);
    for (key, val) in &config.build_args {
      cmd.args(["--build-arg", &format!("{key}={val}")]);
    }
    let status = cmd
      .current_dir(&config.project_dir)
      .status()
      .context("Failed to execute docker build")?;
    if !status.success() {
      anyhow::bail!("Docker build failed with exit code: {:?}", status.code());
    }
    Ok(())
  }

  pub fn run(config: DockerConfig) -> Result<()> {
    let port_str = format!("{}:{}", config.port, config.port);
    let mut cmd = std::process::Command::new("docker");
    cmd.args(["run", "-d", "--rm", "-p", &port_str, &config.image_name]);
    if config.profile == DockerProfile::Dev {
      cmd.args(["-v", ".:/app", "-v", "/app/node_modules"]);
    }
    let status = cmd
      .current_dir(&config.project_dir)
      .status()
      .context("Failed to execute docker run")?;
    if !status.success() {
      anyhow::bail!("Docker run failed with exit code: {:?}", status.code());
    }
    Ok(())
  }

  fn default_infra_services() -> Vec<DockerService> {
    vec![
      DockerService {
        name: "postgres".into(),
        image: "postgres:16-alpine".into(),
        port: 5432,
        env_vars: HashMap::from([
          ("POSTGRES_USER".into(), "klyron".into()),
          ("POSTGRES_PASSWORD".into(), "klyron".into()),
          ("POSTGRES_DB".into(), "klyron".into()),
        ]),
        depends_on: vec![],
        volumes: vec!["postgres_data:/var/lib/postgresql/data".into()],
        health_check: Some(HealthCheckConfig {
          test: vec!["CMD".into(), "pg_isready".into(), "-U".into(), "klyron".into()],
          interval: "10s".into(),
          timeout: "5s".into(),
          retries: 5,
          start_period: "10s".into(),
        }),
        profile: DockerProfile::Prod,
      },
      DockerService {
        name: "redis".into(),
        image: "redis:7-alpine".into(),
        port: 6379,
        env_vars: HashMap::new(),
        depends_on: vec![],
        volumes: vec!["redis_data:/data".into()],
        health_check: Some(HealthCheckConfig {
          test: vec!["CMD".into(), "redis-cli".into(), "ping".into()],
          interval: "10s".into(),
          timeout: "5s".into(),
          retries: 5,
          start_period: "5s".into(),
        }),
        profile: DockerProfile::Prod,
      },
      DockerService {
        name: "mysql".into(),
        image: "mysql:8.0".into(),
        port: 3306,
        env_vars: HashMap::from([
          ("MYSQL_ROOT_PASSWORD".into(), "klyron".into()),
          ("MYSQL_DATABASE".into(), "klyron".into()),
        ]),
        depends_on: vec![],
        volumes: vec!["mysql_data:/var/lib/mysql".into()],
        health_check: Some(HealthCheckConfig {
          test: vec!["CMD".into(), "mysqladmin".into(), "ping".into(), "-h".into(), "localhost".into()],
          interval: "10s".into(),
          timeout: "5s".into(),
          retries: 5,
          start_period: "10s".into(),
        }),
        profile: DockerProfile::Prod,
      },
    ]
  }

  fn detect_project_type(dir: &Path) -> String {
    if dir.join("package.json").exists() {
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
    assert!(content.contains("HEALTHCHECK"));
    assert!(content.contains("--mount=type=cache"));
  }

  #[test]
  fn test_generate_dockerfile_rust() {
    let dir = temp_dir();
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
    DockerManager::generate_dockerfile(&dir).expect("Generate failed");
    let content = fs::read_to_string(dir.join("Dockerfile")).unwrap();
    assert!(content.contains("FROM rust"));
    assert!(content.contains("--mount=type=cache"));
  }

  #[test]
  fn test_generate_dockerignore() {
    let dir = temp_dir();
    DockerManager::generate_dockerignore(&dir).expect("Generate failed");
    let content = fs::read_to_string(dir.join(".dockerignore")).unwrap();
    assert!(content.contains("node_modules"));
    assert!(content.contains(".git"));
  }

  #[test]
  fn test_generate_compose() {
    let dir = temp_dir();
    fs::write(dir.join("package.json"), r#"{"name":"test"}"#).unwrap();
    DockerManager::generate_compose(&dir).expect("Generate failed");
    let content = fs::read_to_string(dir.join("docker-compose.yml")).unwrap();
    assert!(content.contains("services:"));
    assert!(content.contains("postgres"));
    assert!(content.contains("redis"));
    assert!(content.contains("healthcheck"));
  }

  #[test]
  fn test_generate_compose_with_config() {
    let dir = temp_dir();
    fs::write(dir.join("package.json"), r#"{"name":"test"}"#).unwrap();
    let config = DockerConfig {
      image_name: "my-app".into(),
      port: 8080,
      project_dir: dir.clone(),
      profile: DockerProfile::Prod,
      additional_services: vec![],
      build_args: HashMap::new(),
    };
    DockerManager::generate_compose_with_config(&dir, &config).expect("Generate failed");
    let content = fs::read_to_string(dir.join("docker-compose.yml")).unwrap();
    assert!(content.contains("8080:8080"));
    assert!(content.contains("production"));
  }

  #[test]
  fn test_default_infra_services() {
    let services = DockerManager::default_infra_services();
    assert!(services.iter().any(|s| s.name == "postgres"));
    assert!(services.iter().any(|s| s.name == "redis"));
    assert!(services.iter().any(|s| s.name == "mysql"));
    for svc in &services {
      assert!(svc.health_check.is_some());
    }
  }

  #[test]
  fn test_default() {
    let mgr = DockerManager::default();
    let _ = mgr;
  }
}
