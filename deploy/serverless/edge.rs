use super::{Runtime, ServerlessFunction, ServerlessFunctionConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConfig {
    pub regions: Vec<String>,
    pub min_ttl: u32,
    pub max_ttl: u32,
    pub methods: Vec<String>,
    pub cache_key: Vec<String>,
    pub cors: Option<CorsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allow_origin: String,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
    pub max_age: u32,
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            regions: vec!["all".into()],
            min_ttl: 0,
            max_ttl: 86400,
            methods: vec!["GET".into(), "POST".into(), "PUT".into(), "DELETE".into()],
            cache_key: vec!["host".into(), "path".into()],
            cors: None,
        }
    }
}

pub struct EdgeComputing {
    pub config: EdgeConfig,
}

impl EdgeComputing {
    pub fn new(config: EdgeConfig) -> Self {
        Self { config }
    }

    pub fn generate_edge_worker(&self, name: &str) -> String {
        let cors_code = if let Some(ref cors) = self.config.cors {
            format!(
                r#"const corsHeaders = {{
  "Access-Control-Allow-Origin": "{origin}",
  "Access-Control-Allow-Methods": "{methods}",
  "Access-Control-Allow-Headers": "{headers}",
  "Access-Control-Max-Age": "{max_age}",
}};"#,
                origin = cors.allow_origin,
                methods = cors.allow_methods.join(", "),
                headers = cors.allow_headers.join(", "),
                max_age = cors.max_age,
            )
        } else {
            String::new()
        };

        format!(
            r#"// Klyron Edge Worker: {name}
// Deployed regions: {regions}
{cors_code}

const CACHE_CONFIG = {{
  minTtl: {min_ttl},
  maxTtl: {max_ttl},
  cacheKey: [{cache_key}],
}};

export default {{
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {{
    const url = new URL(request.url);
    const method = request.method;
    const cacheKey = CACHE_CONFIG.cacheKey.map(k => {{
      switch (k) {{
        case "host": return url.host;
        case "path": return url.pathname;
        default: return request.headers.get(k) || "";
      }}
    }}).join("|");

    const response = await handleRequest(request, env);
    response.headers.set("X-Edge-Cache-Key", cacheKey);
    response.headers.set("X-Klyron-Edge", "true");
    response.headers.set("X-Klyron-Function", "{name}");
    return response;
  }},
}};

async function handleRequest(request: Request, env: Env): Promise<Response> {{
  const data = {{
    ok: true,
    function: "{name}",
    method: request.method,
    url: request.url,
    timestamp: new Date().toISOString(),
    region: request.cf?.region || "unknown",
    colo: request.cf?.colo || "unknown",
  }};

  return new Response(JSON.stringify(data), {{
    status: 200,
    headers: {{
      "Content-Type": "application/json",
      "Cache-Control": `public, s-maxage=${{CACHE_CONFIG.maxTtl}}`,
    }},
  }});
}}
"#,
            name = name,
            regions = self.config.regions.join(", "),
            cors_code = cors_code,
            min_ttl = self.config.min_ttl,
            max_ttl = self.config.max_ttl,
            cache_key = self.config.cache_key.iter().map(|k| format!("\"{k}\"")).collect::<Vec<_>>().join(", "),
        )
    }
}

impl Default for EdgeComputing {
    fn default() -> Self {
        Self::new(EdgeConfig::default())
    }
}

impl ServerlessFunction for EdgeComputing {
    fn validate(&self, config: &ServerlessFunctionConfig) -> anyhow::Result<()> {
        if config.name.is_empty() {
            anyhow::bail!("edge function name is required");
        }
        Ok(())
    }

    fn generate_handler(&self, config: &ServerlessFunctionConfig, output_dir: &Path) -> anyhow::Result<std::path::PathBuf> {
        let content = self.generate_edge_worker(&config.name);
        let handler_path = output_dir.join(format!("{}.ts", config.name));
        std::fs::create_dir_all(output_dir)?;
        std::fs::write(&handler_path, &content)?;
        Ok(handler_path)
    }

    fn generate_metadata(&self, config: &ServerlessFunctionConfig) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "type": "edge",
            "name": config.name,
            "regions": self.config.regions,
            "min_ttl": self.config.min_ttl,
            "max_ttl": self.config.max_ttl,
            "cache_key": self.config.cache_key,
        }))
    }

    fn bundle(&self, config: &ServerlessFunctionConfig, output_dir: &Path) -> anyhow::Result<std::path::PathBuf> {
        self.generate_handler(config, output_dir)
    }

    fn invoke(&self, config: &ServerlessFunctionConfig, payload: &str) -> anyhow::Result<String> {
        Ok(serde_json::json!({
            "statusCode": 200,
            "body": payload,
            "function": config.name,
            "type": "edge",
        }).to_string())
    }
}
