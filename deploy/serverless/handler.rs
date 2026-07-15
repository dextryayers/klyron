use super::{GenericFunction, Runtime, ServerlessFunction, ServerlessFunctionConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerlessRequest {
    pub http_method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: Option<String>,
    pub request_context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerlessResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub is_base64_encoded: bool,
}

pub struct Handler;

impl Handler {
    pub fn parse_request(raw: &str) -> anyhow::Result<ServerlessRequest> {
        let parsed: ServerlessRequest = serde_json::from_str(raw)
            .map_err(|e| anyhow::anyhow!("Failed to parse serverless request: {e}"))?;
        Ok(parsed)
    }

    pub fn format_response(response: &ServerlessResponse) -> anyhow::Result<String> {
        serde_json::to_string(response)
            .map_err(|e| anyhow::anyhow!("Failed to serialize response: {e}"))
    }

    pub fn handle(config: &ServerlessFunctionConfig, request: &ServerlessRequest) -> anyhow::Result<ServerlessResponse> {
        let generic = GenericFunction;
        generic.validate(config)?;
        let payload = serde_json::to_string(request)?;
        let result = generic.invoke(config, &payload)?;
        Ok(ServerlessResponse {
            status_code: 200,
            headers: HashMap::from([
                ("Content-Type".into(), "application/json".into()),
                ("X-Klyron-Function".into(), config.name.clone()),
            ]),
            body: result,
            is_base64_encoded: false,
        })
    }

    pub fn generate_handler_file(config: &ServerlessFunctionConfig, output_dir: &Path) -> anyhow::Result<()> {
        let generic = GenericFunction;
        generic.generate_handler(config, output_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let raw = r#"{"http_method":"GET","path":"/hello","headers":{"host":"example.com"},"query":{},"body":null}"#;
        let req = Handler::parse_request(raw).unwrap();
        assert_eq!(req.http_method, "GET");
        assert_eq!(req.path, "/hello");
    }

    #[test]
    fn test_format_response() {
        let resp = ServerlessResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: "ok".into(),
            is_base64_encoded: false,
        };
        let json = Handler::format_response(&resp).unwrap();
        assert!(json.contains("200"));
    }
}
