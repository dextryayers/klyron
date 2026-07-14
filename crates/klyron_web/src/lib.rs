use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Headers {
    inner: HashMap<String, Vec<String>>,
}

impl Headers {
    pub fn new() -> Self { Self { inner: HashMap::new() } }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner.get(&name.to_lowercase()).and_then(|v| v.first()).map(|s| s.as_str())
    }

    pub fn get_all(&self, name: &str) -> Vec<&str> {
        self.inner.get(&name.to_lowercase()).map(|v| v.iter().map(|s| s.as_str()).collect()).unwrap_or_default()
    }

    pub fn set(&mut self, name: &str, value: &str) {
        self.inner.insert(name.to_lowercase(), vec![value.to_string()]);
    }

    pub fn append(&mut self, name: &str, value: &str) {
        self.inner.entry(name.to_lowercase()).or_default().push(value.to_string());
    }

    pub fn has(&self, name: &str) -> bool {
        self.inner.contains_key(&name.to_lowercase())
    }

    pub fn remove(&mut self, name: &str) {
        self.inner.remove(&name.to_lowercase());
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.inner.keys().map(|s| s.as_str())
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().flat_map(|(k, vals)| vals.iter().map(move |v| (k.as_str(), v.as_str())))
    }

    pub fn into_hashmap(self) -> HashMap<String, String> {
        self.inner.into_iter().map(|(k, mut v)| (k, v.remove(0))).collect()
    }

    pub fn len(&self) -> usize { self.inner.len() }
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }
}

impl Default for Headers { fn default() -> Self { Self::new() } }

impl From<HashMap<String, String>> for Headers {
    fn from(map: HashMap<String, String>) -> Self {
        let mut h = Headers::new();
        for (k, v) in map { h.set(&k, &v); }
        h
    }
}

#[derive(Debug, Clone)]
pub struct UrlSearchParams {
    params: Vec<(String, String)>,
}

impl UrlSearchParams {
    pub fn new() -> Self { Self { params: Vec::new() } }

    pub fn from_query(query: &str) -> Self {
        let query = query.trim_start_matches('?');
        let mut params = Vec::new();
        for pair in query.split('&').filter(|s| !s.is_empty()) {
            if let Some(idx) = pair.find('=') {
                let k = urlencoding::decode(&pair[..idx]).unwrap_or_else(|_| pair[..idx].into());
                let v = urlencoding::decode(&pair[idx + 1..]).unwrap_or_else(|_| pair[idx + 1..].into());
                params.push((k.into_owned(), v.into_owned()));
            } else {
                let k = urlencoding::decode(pair).unwrap_or_else(|_| pair.into());
                params.push((k.into_owned(), String::new()));
            }
        }
        Self { params }
    }

    pub fn append(&mut self, key: &str, value: &str) {
        self.params.push((key.to_string(), value.to_string()));
    }

    pub fn delete(&mut self, key: &str) {
        self.params.retain(|(k, _)| k != key);
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    pub fn get_all(&self, key: &str) -> Vec<&str> {
        self.params.iter().filter(|(k, _)| k == key).map(|(_, v)| v.as_str()).collect()
    }

    pub fn has(&self, key: &str) -> bool {
        self.params.iter().any(|(k, _)| k == key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.delete(key);
        self.params.push((key.to_string(), value.to_string()));
    }

    pub fn sort(&mut self) {
        self.params.sort_by(|a, b| a.0.cmp(&b.0));
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.params.iter().map(|(k, _)| k.as_str())
    }

    pub fn values(&self) -> impl Iterator<Item = &str> {
        self.params.iter().map(|(_, v)| v.as_str())
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.params.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn to_string(&self) -> String {
        self.params.iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }

    pub fn len(&self) -> usize { self.params.len() }
    pub fn is_empty(&self) -> bool { self.params.is_empty() }
}

impl Default for UrlSearchParams { fn default() -> Self { Self::new() } }

impl std::fmt::Display for UrlSearchParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Url {
    inner: url::Url,
}

impl Url {
    pub fn parse(input: &str) -> anyhow::Result<Self> {
        let inner = url::Url::parse(input)?;
        Ok(Self { inner })
    }

    pub fn parse_with_base(input: &str, base: &str) -> anyhow::Result<Self> {
        let base_url = url::Url::parse(base)?;
        let inner = base_url.join(input)?;
        Ok(Self { inner })
    }

    pub fn protocol(&self) -> &str { self.inner.scheme() }
    pub fn hostname(&self) -> &str { self.inner.host_str().unwrap_or("") }
    pub fn port(&self) -> Option<u16> { self.inner.port() }
    pub fn pathname(&self) -> &str { self.inner.path() }
    pub fn search(&self) -> &str { self.inner.query().map(|q| &*q).unwrap_or("") }
    pub fn hash(&self) -> &str { self.inner.fragment().map(|f| &*f).unwrap_or("") }
    pub fn host(&self) -> &str { self.inner.host_str().unwrap_or("") }
    pub fn origin(&self) -> String {
        format!("{}://{}", self.inner.scheme(), self.inner.host_str().unwrap_or(""))
    }
    pub fn href(&self) -> &str { self.inner.as_str() }

    pub fn search_params(&self) -> UrlSearchParams {
        UrlSearchParams::from_query(self.inner.query().unwrap_or(""))
    }

    pub fn set_protocol(&mut self, protocol: &str) { self.inner.set_scheme(protocol).ok(); }
    pub fn set_hostname(&mut self, hostname: &str) { self.inner.set_host(Some(hostname)).ok(); }
    pub fn set_port(&mut self, port: Option<u16>) { self.inner.set_port(port).ok(); }
    pub fn set_pathname(&mut self, path: &str) { self.inner.set_path(path); }
    pub fn set_search(&mut self, query: &str) {
        let q = query.trim_start_matches('?');
        self.inner.set_query(Some(q));
    }
    pub fn set_hash(&mut self, hash: &str) {
        let h = hash.trim_start_matches('#');
        self.inner.set_fragment(Some(h));
    }

    pub fn join(&self, relative: &str) -> anyhow::Result<Self> {
        let joined = self.inner.join(relative)?;
        Ok(Self { inner: joined })
    }

    pub fn to_string(&self) -> String { self.inner.to_string() }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::str::FromStr for Url {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Self::parse(s) }
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

    pub fn json<T: Serialize>(&mut self, data: &T) -> anyhow::Result<&mut Self> {
        self.body = Some(serde_json::to_string(data)?);
        self.header("Content-Type", "application/json");
        Ok(self)
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

    pub fn status_code(&self) -> u16 { self.status }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
    }
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
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &req.url),
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

    pub fn get(&self, url: &str) -> anyhow::Result<Response> {
        fetch(&Request::new("GET", url))
    }

    pub fn post(&self, url: &str, body: &str) -> anyhow::Result<Response> {
        let mut req = Request::new("POST", url);
        req.body(body);
        fetch(&req)
    }

    pub fn post_json<T: Serialize>(&self, url: &str, data: &T) -> anyhow::Result<Response> {
        let body = serde_json::to_string(data)?;
        let mut req = Request::new("POST", url);
        req.header("Content-Type", "application/json");
        req.body(&body);
        fetch(&req)
    }

    pub fn parse_url(&self, input: &str) -> anyhow::Result<Url> {
        Url::parse(input)
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
    fn test_headers_basic() {
        let mut h = Headers::new();
        h.set("Content-Type", "application/json");
        assert_eq!(h.get("content-type"), Some("application/json"));
        assert!(h.has("Content-Type"));
        assert!(!h.has("X-Nonexistent"));
    }

    #[test]
    fn test_headers_append() {
        let mut h = Headers::new();
        h.append("Set-Cookie", "a=1");
        h.append("Set-Cookie", "b=2");
        assert_eq!(h.get_all("set-cookie"), vec!["a=1", "b=2"]);
    }

    #[test]
    fn test_headers_delete() {
        let mut h = Headers::new();
        h.set("X-Custom", "val");
        assert!(h.has("x-custom"));
        h.remove("X-Custom");
        assert!(!h.has("x-custom"));
    }

    #[test]
    fn test_url_parse() {
        let url = Url::parse("https://user:pass@example.com:8080/path/to?query=1&key=val#frag").unwrap();
        assert_eq!(url.protocol(), "https");
        assert_eq!(url.hostname(), "example.com");
        assert_eq!(url.port(), Some(8080));
        assert_eq!(url.pathname(), "/path/to");
        assert_eq!(url.hash(), "frag");
        assert!(url.href().starts_with("https://"));
    }

    #[test]
    fn test_url_search_params() {
        let url = Url::parse("https://example.com/?foo=1&bar=hello&foo=2").unwrap();
        let sp = url.search_params();
        assert_eq!(sp.get("foo"), Some("1"));
        assert_eq!(sp.get_all("foo"), vec!["1", "2"]);
        assert_eq!(sp.get("bar"), Some("hello"));
        assert!(sp.has("bar"));
        assert!(!sp.has("nonexistent"));
    }

    #[test]
    fn test_url_search_params_manipulate() {
        let mut sp = UrlSearchParams::new();
        sp.append("key", "value1");
        sp.append("key", "value2");
        sp.append("other", "val");
        assert_eq!(sp.len(), 3);
        assert_eq!(sp.get("key"), Some("value1"));
        sp.set("key", "newval");
        assert_eq!(sp.get_all("key"), vec!["newval"]);
        sp.delete("other");
        assert_eq!(sp.len(), 1);
        assert!(!sp.is_empty());
    }

    #[test]
    fn test_url_search_params_sort() {
        let mut sp = UrlSearchParams::from_query("z=1&a=2&m=3");
        sp.sort();
        assert_eq!(sp.to_string(), "a=2&m=3&z=1");
    }

    #[test]
    fn test_url_join() {
        let base = Url::parse("https://example.com/a/b/c").unwrap();
        let joined = base.join("d/e").unwrap();
        assert_eq!(joined.pathname(), "/a/b/d/e");
    }

    #[test]
    fn test_url_setters() {
        let mut url = Url::parse("https://example.com/old").unwrap();
        url.set_pathname("/new");
        assert_eq!(url.pathname(), "/new");
        url.set_search("key=val");
        assert_eq!(url.search(), "key=val");
        url.set_hash("section");
        assert_eq!(url.hash(), "section");
    }

    #[test]
    fn test_url_search_params_from_query() {
        let sp = UrlSearchParams::from_query("?a=1&b=2&c");
        assert_eq!(sp.get("a"), Some("1"));
        assert_eq!(sp.get("b"), Some("2"));
        assert_eq!(sp.get("c"), Some(""));
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

    #[test]
    fn test_response_json() {
        let resp = Response {
            status: 200, status_text: "OK".to_string(), headers: HashMap::new(),
            body: r#"{"name":"test"}"#.to_string(), url: "".to_string(),
        };
        let val: serde_json::Value = resp.json().unwrap();
        assert_eq!(val["name"], "test");
    }
}
