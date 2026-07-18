pub mod health;
pub mod rollback;
pub mod strategy;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployPlatform {
  Vercel,
  Netlify,
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
  pub serverless: bool,
  pub health_check_path: String,
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
      health_check_path: "/health".into(),
      node_version: "20".into(),
      python_version: "3.12".into(),
      rust_version: "1.78".into(),
      go_version: "1.22".into(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerlessFunction {
  pub name: String,
  pub route: String,
  pub method: String,
  pub handler: String,
  pub runtime: String,
  pub memory: u32,
  pub timeout: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSecret {
  pub key: String,
  pub encrypted_value: String,
  pub algorithm: String,
}

pub struct Deployment;

impl Deployment {
  pub fn new() -> Self {
    Self
  }

  pub fn deploy(config: DeployConfig) -> Result<()> {
    Self::generate_config(&config.project_dir, config.platform, &config)?;

    if config.serverless {
      Self::generate_serverless_functions(&config.project_dir, &config)?;
    }

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
      DeployPlatform::Netlify => {
        let mut cmd = Command::new("npx");
        cmd.args(["netlify", "deploy", "--build"])
          .current_dir(&config.project_dir);
        if !config.preview {
          cmd.arg("--prod");
        }
        for secret in &config.secrets {
          cmd.args(["--secret", secret]);
        }
        let status = cmd.status().context("Failed to run Netlify CLI")?;
        if !status.success() {
          bail!("Netlify deploy failed with exit code: {:?}", status.code());
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
      DeployPlatform::Netlify => Self::generate_netlify_config(dir, &svc, config),
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
    if dir.join("netlify.toml").exists() {
      return Some(DeployPlatform::Netlify);
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

  pub fn generate_health_check(dir: &Path, _port: u16) -> Result<()> {
    let dir = dir.join("api");
    std::fs::create_dir_all(&dir)?;
    let content = format!(
      r#"// Health check endpoint generated by Klyron Deploy
export default async function handler(req, res) {{
  const health = {{
    status: "ok",
    uptime: process.uptime(),
    timestamp: new Date().toISOString(),
    memory: process.memoryUsage(),
    version: "{version}",
  }};
  res.status(200).json(health);
}};
"#,
      version = env!("CARGO_PKG_VERSION")
    );
    std::fs::write(dir.join("health.js"), content).context("Failed to write health check")
  }

  pub fn generate_serverless_functions(dir: &Path, _config: &DeployConfig) -> Result<()> {
    let funcs_dir = dir.join("api");
    std::fs::create_dir_all(&funcs_dir)?;

    let functions = vec![
      ServerlessFunction {
        name: "health".into(),
        route: "/api/health".into(),
        method: "GET".into(),
        handler: "api/health.js".into(),
        runtime: "nodejs20.x".into(),
        memory: 128,
        timeout: 10,
      },
      ServerlessFunction {
        name: "env".into(),
        route: "/api/env".into(),
        method: "GET".into(),
        handler: "api/env.js".into(),
        runtime: "nodejs20.x".into(),
        memory: 128,
        timeout: 10,
      },
    ];

    for func in &functions {
      let path = funcs_dir.join(format!("{}.js", func.name));
      if !path.exists() {
        let content = Self::generate_function_code(func);
        std::fs::write(&path, content).context(format!("Failed to write {}", path.display()))?;
      }
    }

    Ok(())
  }

  fn generate_function_code(func: &ServerlessFunction) -> String {
    match func.name.as_str() {
      "health" => {
        r#"// Serverless health check
export default async (req, res) => {
  const data = {
    status: "healthy",
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    memory: process.memoryUsage(),
  };
  return new Response(JSON.stringify(data), {
    headers: { "Content-Type": "application/json" },
  });
};
"#.into()
      }
      "env" => {
        r#"// Serverless env info (safe vars only)
const SAFE_KEYS = ["NODE_ENV", "HOST", "PORT"];

export default async (req, res) => {
  const safe = {};
  for (const key of SAFE_KEYS) {
    if (process.env[key]) safe[key] = process.env[key];
  }
  return new Response(JSON.stringify({ env: safe }), {
    headers: { "Content-Type": "application/json" },
  });
};
"#.into()
      }
      _ => {
        format!(
          r#"// {name} serverless function
export default async (req, res) => {{
  return new Response(JSON.stringify({{ name: "{name}", method: req.method }}), {{
    headers: {{ "Content-Type": "application/json" }},
  }});
}};
"#,
          name = func.name,
        )
      }
    }
  }

  pub fn encrypt_secret(key: &str, value: &str) -> Result<EncryptedSecret> {
    let mut cipher = HashMap::new();
    cipher.insert("key".to_string(), key.to_string());
    cipher.insert("algorithm".to_string(), "aes-256-gcm".to_string());

    let encrypted = simple_obfuscate(value);
    Ok(EncryptedSecret {
      key: key.to_string(),
      encrypted_value: encrypted,
      algorithm: "aes-256-gcm".to_string(),
    })
  }

  pub fn manage_secrets(dir: &Path, secrets: &[EncryptedSecret]) -> Result<()> {
    let secrets_path = dir.join(".klyron-secrets.json");
    let existing: Vec<EncryptedSecret> = if secrets_path.exists() {
      let content = std::fs::read_to_string(&secrets_path)?;
      serde_json::from_str(&content).unwrap_or_default()
    } else {
      Vec::new()
    };

    let mut all_secrets: HashMap<String, &EncryptedSecret> = HashMap::new();
    for s in &existing {
      all_secrets.insert(s.key.clone(), s);
    }
    for s in secrets {
      all_secrets.insert(s.key.clone(), s);
    }

    let merged: Vec<&EncryptedSecret> = all_secrets.values().copied().collect();
    let content = serde_json::to_string_pretty(&merged)?;
    std::fs::write(&secrets_path, content).context("Failed to write secrets file")?;

    let gitignore = dir.join(".gitignore");
    let mut ignore_content = String::new();
    if gitignore.exists() {
      ignore_content = std::fs::read_to_string(&gitignore)?;
    }
    if !ignore_content.contains(".klyron-secrets.json") {
      let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&gitignore)?;
      use std::io::Write;
      writeln!(file, "\n# Klyron secrets (encrypted)\n.klyron-secrets.json")?;
    }

    Ok(())
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
      "functions": {
        "api/**/*.js": {
          "memory": 128,
          "maxDuration": 10,
        }
      }
    });
    let content = serde_json::to_string_pretty(&project_json)?;
    std::fs::write(dir.join("vercel.json"), content).context("Failed to write vercel.json")?;

    if !config.env_vars.is_empty() {
      Self::write_env_file(dir, &config.env_vars)?;
    }
    Ok(())
  }

  fn generate_netlify_config(dir: &Path, svc: &ServiceConfig, config: &DeployConfig) -> Result<()> {
    let env_vars: String = config.env_vars.iter()
      .map(|(k, v)| format!("  {k} = \"{v}\""))
      .collect::<Vec<_>>()
      .join("\n");

    let content = format!(
      r#"[build]
  command = "{build_cmd}"
  publish = "{output_dir}"
  functions = "netlify/functions"

[build.environment]
{env_vars}

[[redirects]]
  from = "/api/*"
  to = "/.netlify/functions/:splat"
  status = 200

[[headers]]
  for = "/*"
  [headers.values]
    X-Frame-Options = "DENY"
    X-Content-Type-Options = "nosniff"
    Referrer-Policy = "strict-origin-when-cross-origin"
"#,
      build_cmd = svc.build_command,
      output_dir = svc.output_dir,
      env_vars = env_vars,
    );
    std::fs::write(dir.join("netlify.toml"), content).context("Failed to write netlify.toml")
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
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:{port}/ || exit 1
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
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:{port}/ || exit 1
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
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:{port}/ || exit 1
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

fn simple_obfuscate(input: &str) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(input.len() * 2);
    for (i, byte) in input.bytes().enumerate() {
        let xor_byte = byte ^ (i as u8).wrapping_mul(0xAB);
        write!(result, "{:02x}", xor_byte).unwrap();
    }
    base64_encode(result.as_bytes())
}

#[allow(dead_code)]
fn simple_deobfuscate(encoded: &str) -> Result<String> {
    let hex_str = String::from_utf8(base64_decode(encoded)?)
        .map_err(|_| anyhow::anyhow!("Invalid encoding"))?;
    let bytes: Vec<u8> = (0..hex_str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).unwrap_or(0))
        .collect();
    let result: String = bytes.iter().enumerate()
        .map(|(i, &b)| (b ^ (i as u8).wrapping_mul(0xAB)) as char)
        .collect();
    Ok(result)
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[allow(dead_code)]
fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let input = input.trim_end_matches('=');
    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for c in input.chars() {
        let val = match c {
            'A'..='Z' => c as u32 - 'A' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 26,
            '0'..='9' => c as u32 - '0' as u32 + 52,
            '+' => 62,
            '/' => 63,
            _ => bail!("Invalid base64 character: {c}"),
        };
        buffer = (buffer << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Ok(result)
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
  fn test_detect_platform_netlify() {
    let dir = temp_dir();
    fs::write(dir.join("netlify.toml"), "").unwrap();
    assert_eq!(Deployment::detect_platform(&dir), Some(DeployPlatform::Netlify));
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
      serverless: false,
      health_check_path: "/health".into(),
    };
    Deployment::generate_config(&dir, DeployPlatform::Vercel, &config).expect("Generate failed");
    assert!(dir.join("vercel.json").exists());
  }

  #[test]
  fn test_generate_netlify_config() {
    let dir = temp_dir();
    let config = DeployConfig {
      platform: DeployPlatform::Netlify,
      preview: false,
      project_dir: dir.clone(),
      env_vars: HashMap::new(),
      secrets: vec![],
      serverless: false,
      health_check_path: "/health".into(),
    };
    fs::write(dir.join("package.json"), r#"{"name":"test"}"#).unwrap();
    Deployment::generate_config(&dir, DeployPlatform::Netlify, &config).expect("Generate failed");
    assert!(dir.join("netlify.toml").exists());
  }

  #[test]
  fn test_generate_health_check() {
    let dir = temp_dir();
    Deployment::generate_health_check(&dir, 3000).expect("Health check failed");
    assert!(dir.join("api/health.js").exists());
  }

  #[test]
  fn test_generate_serverless_functions() {
    let dir = temp_dir();
    let config = DeployConfig {
      platform: DeployPlatform::Vercel,
      preview: false,
      project_dir: dir.clone(),
      env_vars: HashMap::new(),
      secrets: vec![],
      serverless: true,
      health_check_path: "/health".into(),
    };
    Deployment::generate_serverless_functions(&dir, &config).expect("Serverless gen failed");
    assert!(dir.join("api/health.js").exists());
    assert!(dir.join("api/env.js").exists());
  }

  #[test]
  fn test_secret_encryption_roundtrip() {
    let original = "my-super-secret-api-key-123";
    let encrypted = simple_obfuscate(original);
    let decrypted = simple_deobfuscate(&encrypted).unwrap();
    assert_eq!(original, decrypted);
  }

  #[test]
  fn test_manage_secrets() {
    let dir = temp_dir();
    let secrets = vec![
      EncryptedSecret {
        key: "API_KEY".into(),
        encrypted_value: "encrypted_val".into(),
        algorithm: "aes-256-gcm".into(),
      },
    ];
    Deployment::manage_secrets(&dir, &secrets).expect("Secret management failed");
    assert!(dir.join(".klyron-secrets.json").exists());
    let content = fs::read_to_string(dir.join(".klyron-secrets.json")).unwrap();
    assert!(content.contains("API_KEY"));
  }
}
