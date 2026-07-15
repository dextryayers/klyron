use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub mod edge;
pub mod handler;
pub mod wasm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Runtime {
    Nodejs18,
    Nodejs20,
    Nodejs22,
    Python39,
    Python310,
    Python311,
    Python312,
    Rust,
    Go,
    Wasm,
    Deno,
    Bun,
}

impl Runtime {
    pub fn identifier(&self) -> &str {
        match self {
            Self::Nodejs18 => "nodejs18.x",
            Self::Nodejs20 => "nodejs20.x",
            Self::Nodejs22 => "nodejs22.x",
            Self::Python39 => "python3.9",
            Self::Python310 => "python3.10",
            Self::Python311 => "python3.11",
            Self::Python312 => "python3.12",
            Self::Rust => "provided.al2023",
            Self::Go => "go1.x",
            Self::Wasm => "wasm",
            Self::Deno => "deno",
            Self::Bun => "bun",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerlessFunctionConfig {
    pub name: String,
    pub route: String,
    pub method: String,
    pub handler: String,
    pub runtime: Runtime,
    pub memory_mb: u32,
    pub timeout_seconds: u32,
    pub env_vars: HashMap<String, String>,
    pub layers: Vec<String>,
    pub vpc_config: Option<VpcConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpcConfig {
    pub security_group_ids: Vec<String>,
    pub subnet_ids: Vec<String>,
}

pub trait ServerlessFunction {
    fn validate(&self, config: &ServerlessFunctionConfig) -> anyhow::Result<()>;
    fn generate_handler(&self, config: &ServerlessFunctionConfig, output_dir: &Path) -> anyhow::Result<PathBuf>;
    fn generate_metadata(&self, config: &ServerlessFunctionConfig) -> anyhow::Result<serde_json::Value>;
    fn bundle(&self, config: &ServerlessFunctionConfig, output_dir: &Path) -> anyhow::Result<PathBuf>;
    fn invoke(&self, config: &ServerlessFunctionConfig, payload: &str) -> anyhow::Result<String>;
}

pub struct GenericFunction;

impl GenericFunction {
    pub fn generate_js_handler(name: &str, route: &str) -> String {
        format!(
            r#"// Klyron Serverless Function: {name}
// Route: {route}
export default async (request, context) => {{
  const method = request.method;
  const url = new URL(request.url);
  const headers = Object.fromEntries(request.headers);

  const body = {{
    ok: true,
    function: "{name}",
    method,
    path: url.pathname,
    timestamp: new Date().toISOString(),
    env: {{
      NODE_ENV: process.env.NODE_ENV || "development",
    }},
  }};

  return new Response(JSON.stringify(body), {{
    status: 200,
    headers: {{ "Content-Type": "application/json" }},
  }});
}};
"#,
            name = name,
            route = route,
        )
    }

    pub fn generate_ts_handler(name: &str, route: &str) -> String {
        format!(
            r#"// Klyron Serverless Function: {name}
// Route: {route}
import type {{ Request, Response }} from "@cloudflare/workers-types";

interface Env {{
  NODE_ENV?: string;
  [key: string]: unknown;
}}

interface FunctionResponse {{
  ok: boolean;
  function: string;
  method: string;
  path: string;
  timestamp: string;
  env: {{ NODE_ENV: string }};
}}

export default {{
  async fetch(request: Request, env: Env): Promise<Response> {{
    const method = request.method;
    const url = new URL(request.url);

    const body: FunctionResponse = {{
      ok: true,
      function: "{name}",
      method,
      path: url.pathname,
      timestamp: new Date().toISOString(),
      env: {{
        NODE_ENV: env.NODE_ENV || "development",
      }},
    }};

    return new Response(JSON.stringify(body), {{
      status: 200,
      headers: {{ "Content-Type": "application/json" }},
    }});
  }},
}};
"#,
            name = name,
            route = route,
        )
    }

    pub fn generate_python_handler(name: &str) -> String {
        format!(
            r#"""Klyron Serverless Function: {name}"""
import json
import os
from datetime import datetime, timezone

def handler(event, context):
    return {{
        "statusCode": 200,
        "headers": {{"Content-Type": "application/json"}},
        "body": json.dumps({{
            "ok": True,
            "function": "{name}",
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "env": {{"NODE_ENV": os.environ.get("NODE_ENV", "development")}},
        }}),
    }}
"#,
            name = name,
        )
    }
}

impl ServerlessFunction for GenericFunction {
    fn validate(&self, config: &ServerlessFunctionConfig) -> anyhow::Result<()> {
        if config.name.is_empty() {
            anyhow::bail!("function name is required");
        }
        if config.handler.is_empty() {
            anyhow::bail!("handler path is required");
        }
        if config.memory_mb < 128 || config.memory_mb > 10240 {
            anyhow::bail!("memory_mb must be between 128 and 10240");
        }
        if config.timeout_seconds < 1 || config.timeout_seconds > 900 {
            anyhow::bail!("timeout_seconds must be between 1 and 900");
        }
        Ok(())
    }

    fn generate_handler(&self, config: &ServerlessFunctionConfig, output_dir: &Path) -> anyhow::Result<PathBuf> {
        let handler_content = match config.runtime {
            Runtime::Nodejs18 | Runtime::Nodejs20 | Runtime::Nodejs22 | Runtime::Deno | Runtime::Bun => {
                Self::generate_js_handler(&config.name, &config.route)
            }
            Runtime::Python39 | Runtime::Python310 | Runtime::Python311 | Runtime::Python312 => {
                Self::generate_python_handler(&config.name)
            }
            _ => Self::generate_js_handler(&config.name, &config.route),
        };
        let ext = match config.runtime {
            Runtime::Python39 | Runtime::Python310 | Runtime::Python311 | Runtime::Python312 => "py",
            _ => "js",
        };
        let handler_path = output_dir.join(format!("{}.{}", config.name, ext));
        std::fs::create_dir_all(output_dir)?;
        std::fs::write(&handler_path, &handler_content)?;
        Ok(handler_path)
    }

    fn generate_metadata(&self, config: &ServerlessFunctionConfig) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "name": config.name,
            "route": config.route,
            "method": config.method,
            "handler": config.handler,
            "runtime": config.runtime.identifier(),
            "memory": config.memory_mb,
            "timeout": config.timeout_seconds,
            "layers": config.layers,
        }))
    }

    fn bundle(&self, config: &ServerlessFunctionConfig, output_dir: &Path) -> anyhow::Result<PathBuf> {
        let handler_path = self.generate_handler(config, output_dir)?;
        Ok(handler_path)
    }

    fn invoke(&self, config: &ServerlessFunctionConfig, payload: &str) -> anyhow::Result<String> {
        Ok(serde_json::json!({
            "statusCode": 200,
            "body": payload,
            "function": config.name,
        }).to_string())
    }
}
