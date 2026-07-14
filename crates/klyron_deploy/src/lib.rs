use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployPlatform {
  Vercel,
  Cloudflare,
  Railway,
  Fly,
  Docker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
  pub platform: DeployPlatform,
  pub preview: bool,
  pub project_dir: PathBuf,
  pub env_vars: HashMap<String, String>,
  pub secrets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
  pub name: String,
  pub port: u16,
  pub build_command: String,
  pub start_command: String,
  pub install_command: String,
  pub output_dir: String,
  pub health_check_path: String,
  pub node_version: String,
  pub python_version: String,
  pub rust_version: String,
  pub go_version: String,
}

impl Default for ServiceConfig {
  fn default() -> Self {
    ServiceConfig {
      name: "app".into(),
      port: 3000,
      build_command: "npm run build".into(),
      start_command: "npm start".into(),
      install_command: "npm install".into(),
      output_dir: "dist".into(),
      health_check_path: "/".into(),
      node_version: "20".into(),
      python_version: "3.12".into(),
      rust_version: "1.78".into(),
      go_version: "1.22".into(),
    }
  }
}

pub struct Deployment;

impl Deployment {
  pub fn new() -> Self {
    Self
  }

  pub fn deploy(config: DeployConfig) -> Result<()> {
    Self::generate_config(&config.project_dir, config.platform, &config)?;

    match config.platform {
      DeployPlatform::Vercel => {
        let mut cmd = Command::new("npx");
        cmd.args(["vercel", "--prod"])
          .arg("--yes")
          .current_dir(&config.project_dir);
        if config.preview {
          cmd.arg("--prebuilt");
        }
        for (key, val) in &config.env_vars {
          cmd.env(key, val);
        }
        let status = cmd.status().context("Failed to run Vercel CLI")?;
        if !status.success() {
          bail!("Vercel deploy failed with exit code: {:?}", status.code());
        }
      }
      DeployPlatform::Cloudflare => {
        let mut cmd = Command::new("npx");
        cmd.args(["wrangler", "deploy"])
          .current_dir(&config.project_dir);
        if config.preview {
          cmd.arg("--preview");
        }
        for secret in &config.secrets {
          cmd.args(["--secret", secret]);
        }
        let status = cmd.status().context("Failed to run Wrangler CLI")?;
        if !status.success() {
          bail!("Cloudflare deploy failed with exit code: {:?}", status.code());
        }
      }
      DeployPlatform::Railway => {
        let mut cmd = Command::new("railway");
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
        let status = Command::new("flyctl")
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
          config.project_dir.file_name().unwrap_or_default().to_string_lossy()
        );
        let build_status = Command::new("docker")
          .args(["build", "-t", &image_name, "."])
          .current_dir(&config.project_dir)
          .status()
          .context("Failed to run Docker build")?;
        if !build_status.success() {
          bail!("Docker build failed with exit code: {:?}", build_status.code());
        }
        let run_status = Command::new("docker")
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

  pub fn generate_config(dir: &Path, platform: DeployPlatform, config: &DeployConfig) -> Result<()> {
    let svc = Self::detect_service_config(dir);
    match platform {
      DeployPlatform::Vercel => Self::generate_vercel_config(dir, &svc, config),
      DeployPlatform::Cloudflare => Self::generate_cloudflare_config(dir, &svc, config),
      DeployPlatform::Railway => Self::generate_railway_config(dir, &svc, config),
      DeployPlatform::Fly => Self::generate_fly_config(dir, &svc, config),
      DeployPlatform::Docker => Self::generate_docker_config(dir, &svc, config),
    }
  }

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

  pub fn write_env_file(dir: &Path, env_vars: &HashMap<String, String>) -> Result<()> {
    let mut content = String::new();
    for (key, val) in env_vars {
      content.push_str(&format!("{key}={val}\n"));
    }
    std::fs::write(dir.join(".env"), content.trim()).context("Failed to write .env")
  }

  pub fn write_env_example(dir: &Path, env_vars: &HashMap<String, String>) -> Result<()> {
    let mut content = String::new();
    for key in env_vars.keys() {
      content.push_str(&format!("{key}=\n"));
    }
    std::fs::write(dir.join(".env.example"), content.trim()).context("Failed to write .env.example")
  }

  fn detect_service_config(dir: &Path) -> ServiceConfig {
    let name = dir.file_name().unwrap_or_default().to_string_lossy().to_string();
    let mut svc = ServiceConfig {
      name,
      ..Default::default()
    };

    if dir.join("Cargo.toml").exists() {
      svc.port = 8080;
      svc.build_command = "cargo build --release".into();
      svc.start_command = "./target/release/app".into();
      svc.install_command = String::new();
      svc.output_dir = "target/release".into();
    } else if dir.join("go.mod").exists() {
      svc.port = 8080;
      svc.build_command = "go build -o dist/app".into();
      svc.start_command = "./dist/app".into();
      svc.output_dir = "dist".into();
    } else if dir.join("requirements.txt").exists() || dir.join("setup.py").exists() {
      svc.port = 8000;
      svc.build_command = "pip install -r requirements.txt".into();
      svc.start_command = "python main.py".into();
      svc.output_dir = ".".into();
    } else if dir.join("composer.json").exists() {
      svc.port = 8000;
      svc.build_command = "composer install --no-dev".into();
      svc.start_command = "php -S 0.0.0.0:8000 -t public".into();
      svc.output_dir = "public".into();
    }

    if let Ok(content) = std::fs::read_to_string(dir.join("package.json")) {
      if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
          if deps.contains_key("next") {
            svc.build_command = "next build".into();
            svc.start_command = "next start".into();
            svc.output_dir = ".next".into();
          }
        }
      }
    }

    svc
  }

  fn generate_vercel_config(dir: &Path, svc: &ServiceConfig, config: &DeployConfig) -> Result<()> {
    let project_json = serde_json::json!({
      "framework": svc.name,
      "buildCommand": svc.build_command,
      "outputDirectory": svc.output_dir,
      "installCommand": svc.install_command,
      "devCommand": "npm run dev",
    });
    let content = serde_json::to_string_pretty(&project_json)?;
    std::fs::write(dir.join("vercel.json"), content).context("Failed to write vercel.json")?;

    if !config.env_vars.is_empty() {
      Self::write_env_file(dir, &config.env_vars)?;
    }
    Ok(())
  }

  fn generate_cloudflare_config(dir: &Path, svc: &ServiceConfig, config: &DeployConfig) -> Result<()> {
    let content = format!(
      r#"name = "{name}"
compatibility_date = "2024-12-01"
main = "src/index.js"

[build]
command = "{build_cmd}"

[env]
{env_vars}

[secrets]
{secrets}
"#,
      name = svc.name,
      build_cmd = svc.build_command,
      env_vars = config.env_vars.iter().map(|(k, v)| format!("{k} = \"{v}\"")).collect::<Vec<_>>().join("\n"),
      secrets = config.secrets.iter().map(|s| format!("{s} = \"\"")).collect::<Vec<_>>().join("\n"),
    );
    std::fs::write(dir.join("wrangler.toml"), content).context("Failed to write wrangler.toml")
  }

  fn generate_railway_config(dir: &Path, svc: &ServiceConfig, config: &DeployConfig) -> Result<()> {
    let rail_json = serde_json::json!({
      "build": {
        "builder": "NIXPACKS",
        "buildCommand": svc.build_command,
      },
      "deploy": {
        "startCommand": svc.start_command,
        "healthcheckPath": svc.health_check_path,
        "restartPolicyType": "always",
      },
      "env": config.env_vars,
    });
    let content = serde_json::to_string_pretty(&rail_json)?;
    std::fs::write(dir.join("railway.json"), content).context("Failed to write railway.json")
  }

  fn generate_fly_config(dir: &Path, svc: &ServiceConfig, config: &DeployConfig) -> Result<()> {
    let env_section: String = config.env_vars.iter()
      .map(|(k, v)| format!("  {k} = \"{v}\""))
      .collect::<Vec<_>>()
      .join("\n");

    let content = format!(
      r#"app = "{name}"
primary_region = "iad"

[build]
  builder = "heroku/buildpacks:20"

[env]
{env_section}

[http_service]
  internal_port = {port}
  force_https = true
  healthcheck = {{ path = "{health}", interval = "10s", timeout = "5s" }}
"#,
      name = svc.name,
      port = svc.port,
      health = svc.health_check_path,
      env_section = env_section,
    );
    std::fs::write(dir.join("fly.toml"), content).context("Failed to write fly.toml")
  }

  fn generate_docker_config(dir: &Path, svc: &ServiceConfig, _config: &DeployConfig) -> Result<()> {
    let dockerfile = if dir.join("Cargo.toml").exists() {
      format!(
        r#"FROM rust:{rust_ver} AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {{}}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/app /usr/local/bin/app
EXPOSE {port}
CMD ["app"]
"#,
        rust_ver = svc.rust_version,
        port = svc.port,
      )
    } else if dir.join("go.mod").exists() {
      format!(
        r#"FROM golang:{go_ver} AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o /app/server .

FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder /app/server /server
EXPOSE {port}
CMD ["/server"]
"#,
        go_ver = svc.go_version,
        port = svc.port,
      )
    } else {
      format!(
        r#"FROM node:{node_ver}-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN {build_cmd}

FROM node:{node_ver}-alpine
WORKDIR /app
COPY --from=builder /app/{output_dir} ./{output_dir}
COPY --from=builder /app/node_modules ./node_modules
COPY package*.json ./
EXPOSE {port}
CMD {start_cmd}
"#,
        node_ver = svc.node_version,
        build_cmd = svc.build_command,
        output_dir = svc.output_dir,
        port = svc.port,
        start_cmd = svc.start_command,
      )
    };
    std::fs::write(dir.join("Dockerfile"), dockerfile).context("Failed to write Dockerfile")
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
    assert_eq!(Deployment::detect_platform(&dir), Some(DeployPlatform::Cloudflare));
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
    let config = DeployConfig {
      platform: DeployPlatform::Vercel,
      preview: false,
      project_dir: dir.clone(),
      env_vars: HashMap::new(),
      secrets: vec![],
    };
    Deployment::generate_config(&dir, DeployPlatform::Vercel, &config).expect("Generate failed");
    assert!(dir.join("vercel.json").exists());
  }

  #[test]
  fn test_generate_cloudflare_config() {
    let dir = temp_dir();
    let config = DeployConfig {
      platform: DeployPlatform::Cloudflare,
      preview: false,
      project_dir: dir.clone(),
      env_vars: HashMap::new(),
      secrets: vec![],
    };
    fs::write(dir.join("package.json"), r#"{"name":"test"}"#).unwrap();
    Deployment::generate_config(&dir, DeployPlatform::Cloudflare, &config).expect("Generate failed");
    assert!(dir.join("wrangler.toml").exists());
  }

  #[test]
  fn test_generate_docker_config() {
    let dir = temp_dir();
    let config = DeployConfig {
      platform: DeployPlatform::Docker,
      preview: false,
      project_dir: dir.clone(),
      env_vars: HashMap::new(),
      secrets: vec![],
    };
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    Deployment::generate_config(&dir, DeployPlatform::Docker, &config).expect("Generate failed");
    assert!(dir.join("Dockerfile").exists());
    let content = fs::read_to_string(dir.join("Dockerfile")).unwrap();
    assert!(content.contains("FROM rust"));
  }

  #[test]
  fn test_env_file_management() {
    let dir = temp_dir();
    let mut vars = HashMap::new();
    vars.insert("DATABASE_URL".into(), "postgres://localhost".into());
    vars.insert("API_KEY".into(), "secret123".into());
    Deployment::write_env_file(&dir, &vars).unwrap();
    assert!(dir.join(".env").exists());
    Deployment::write_env_example(&dir, &vars).unwrap();
    assert!(dir.join(".env.example").exists());
  }

  #[test]
  fn test_serialization() {
    let config = DeployConfig {
      platform: DeployPlatform::Vercel,
      preview: true,
      project_dir: PathBuf::from("/tmp/test"),
      env_vars: HashMap::new(),
      secrets: vec![],
    };
    let json = serde_json::to_string(&config).unwrap();
    let back: DeployConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.platform, DeployPlatform::Vercel);
    assert!(back.preview);
  }
}
