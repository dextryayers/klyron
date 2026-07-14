use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Headers {
    inner: HashMap<String, String>,
}

impl Headers {
    pub fn new() -> Self { Self { inner: HashMap::new() } }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner.get(&name.to_lowercase()).map(|s| s.as_str())
    }

    pub fn set(&mut self, name: &str, value: &str) {
        self.inner.insert(name.to_lowercase(), value.to_string());
    }

    pub fn remove(&mut self, name: &str) {
        self.inner.remove(&name.to_lowercase());
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
}

impl Request {
    pub fn new(method: &str, url: &str) -> Self {
        Self { method: method.to_string(), url: url.to_string(), headers: HashMap::new(), body: None }
    }

    pub fn header(&mut self, name: &str, value: &str) -> &mut Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn body(&mut self, body: &str) -> &mut Self {
        self.body = Some(body.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub url: String,
}

impl Response {
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<T> {
        Ok(serde_json::from_str(&self.body)?)
    }

    pub fn text(&self) -> &str { &self.body }

    pub fn ok(&self) -> bool { self.status >= 200 && self.status < 300 }
}

pub fn fetch(req: &Request) -> anyhow::Result<Response> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let mut http_req = match req.method.to_uppercase().as_str() {
        "GET" => client.get(&req.url),
        "POST" => client.post(&req.url),
        "PUT" => client.put(&req.url),
        "DELETE" => client.delete(&req.url),
        "PATCH" => client.patch(&req.url),
        "HEAD" => client.head(&req.url),
        m => anyhow::bail!("Unsupported HTTP method: {m}"),
    };

    for (k, v) in &req.headers {
        http_req = http_req.header(k.as_str(), v.as_str());
    }

    if let Some(body) = &req.body {
        http_req = http_req.body(body.clone());
    }

    let resp = http_req.send()?;
    let status = resp.status().as_u16();
    let status_text = resp.status().canonical_reason().unwrap_or("Unknown").to_string();
    let url = resp.url().to_string();
    let headers = resp.headers().iter().map(|(k, v)| {
        (k.as_str().to_string(), v.to_str().unwrap_or("").to_string())
    }).collect();
    let body = resp.text()?;

    Ok(Response { status, status_text, headers, body, url })
}

pub fn fetch_url(url: &str) -> anyhow::Result<Response> {
    let req = Request::new("GET", url);
    fetch(&req)
}

#[derive(Debug, Clone)]
pub struct WebApi;

impl WebApi {
    pub fn new() -> Self { Self }

    pub fn fetch(&self, url: &str) -> anyhow::Result<Response> {
        fetch_url(url)
    }

    pub fn request(&self, req: &Request) -> anyhow::Result<Response> {
        fetch(req)
    }
}

pub fn encode_uri_component(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

pub fn decode_uri_component(s: &str) -> String {
    url::form_urlencoded::parse(s.as_bytes())
        .map(|(k, v)| [k, v].concat())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_headers() {
        let mut h = Headers::new();
        h.set("Content-Type", "application/json");
        assert_eq!(h.get("content-type"), Some("application/json"));
    }

    #[test]
    fn test_encode_decode() {
        let s = "hello world";
        let encoded = encode_uri_component(s);
        assert_eq!(encoded, "hello+world");
    }

    #[test]
    fn test_response_ok() {
        let resp = Response {
            status: 200, status_text: "OK".to_string(), headers: HashMap::new(),
            body: "{}".to_string(), url: "http://example.com".to_string(),
        };
        assert!(resp.ok());
    }

    #[test]
    fn test_response_not_ok() {
        let resp = Response {
            status: 404, status_text: "Not Found".to_string(), headers: HashMap::new(),
            body: "".to_string(), url: "http://example.com".to_string(),
        };
        assert!(!resp.ok());
    }
}
