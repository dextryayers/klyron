use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Headers {
    inner: HashMap<String, Vec<String>>,
}

impl Headers {
    pub fn new() -> Self { Self { inner: HashMap::new() } }
    pub fn get(&self, name: &str) -> Option<&str> { self.inner.get(&name.to_lowercase()).and_then(|v| v.first()).map(|s| s.as_str()) }
    pub fn get_all(&self, name: &str) -> Vec<&str> { self.inner.get(&name.to_lowercase()).map(|v| v.iter().map(|s| s.as_str()).collect()).unwrap_or_default() }
    pub fn set(&mut self, name: &str, value: &str) { self.inner.insert(name.to_lowercase(), vec![value.to_string()]); }
    pub fn append(&mut self, name: &str, value: &str) { self.inner.entry(name.to_lowercase()).or_default().push(value.to_string()); }
    pub fn has(&self, name: &str) -> bool { self.inner.contains_key(&name.to_lowercase()) }
    pub fn remove(&mut self, name: &str) { self.inner.remove(&name.to_lowercase()); }
    pub fn keys(&self) -> impl Iterator<Item = &str> { self.inner.keys().map(|s| s.as_str()) }
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> { self.inner.iter().flat_map(|(k, vals)| vals.iter().map(move |v| (k.as_str(), v.as_str()))) }
    pub fn into_hashmap(self) -> HashMap<String, String> { self.inner.into_iter().map(|(k, mut v)| (k, v.remove(0))).collect() }
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
    pub fn from_query(query: &str) -> Self { let q = query.trim_start_matches('?'); let mut p = Self::new(); for pair in q.split('&').filter(|s| !s.is_empty()) { if let Some(idx) = pair.find('=') { p.params.push((urlencoding::decode(&pair[..idx]).unwrap_or_default().into_owned(), urlencoding::decode(&pair[idx+1..]).unwrap_or_default().into_owned())); } else { p.params.push((urlencoding::decode(pair).unwrap_or_default().into_owned(), String::new())); } } p }
    pub fn append(&mut self, key: &str, value: &str) { self.params.push((key.to_string(), value.to_string())); }
    pub fn delete(&mut self, key: &str) { self.params.retain(|(k, _)| k != key); }
    pub fn get(&self, key: &str) -> Option<&str> { self.params.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()) }
    pub fn get_all(&self, key: &str) -> Vec<&str> { self.params.iter().filter(|(k, _)| k == key).map(|(_, v)| v.as_str()).collect() }
    pub fn has(&self, key: &str) -> bool { self.params.iter().any(|(k, _)| k == key) }
    pub fn set(&mut self, key: &str, value: &str) { self.delete(key); self.params.push((key.to_string(), value.to_string())); }
    pub fn sort(&mut self) { self.params.sort_by(|a, b| a.0.cmp(&b.0)); }
    pub fn keys(&self) -> impl Iterator<Item = &str> { self.params.iter().map(|(k, _)| k.as_str()) }
    pub fn values(&self) -> impl Iterator<Item = &str> { self.params.iter().map(|(_, v)| v.as_str()) }
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str)> { self.params.iter().map(|(k, v)| (k.as_str(), v.as_str())) }
    pub fn to_string(&self) -> String { self.params.iter().map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v))).collect::<Vec<_>>().join("&") }
    pub fn len(&self) -> usize { self.params.len() }
    pub fn is_empty(&self) -> bool { self.params.is_empty() }
}
impl Default for UrlSearchParams { fn default() -> Self { Self::new() } }
impl std::fmt::Display for UrlSearchParams { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.to_string()) } }

#[derive(Debug, Clone)]
pub struct Url {
    inner: url::Url,
}
impl Url {
    pub fn parse(input: &str) -> anyhow::Result<Self> { Ok(Self { inner: url::Url::parse(input)? }) }
    pub fn parse_with_base(input: &str, base: &str) -> anyhow::Result<Self> { Ok(Self { inner: url::Url::parse(base)?.join(input)? }) }
    pub fn protocol(&self) -> &str { self.inner.scheme() }
    pub fn hostname(&self) -> &str { self.inner.host_str().unwrap_or("") }
    pub fn port(&self) -> Option<u16> { self.inner.port() }
    pub fn pathname(&self) -> &str { self.inner.path() }
    pub fn search(&self) -> &str { self.inner.query().unwrap_or("") }
    pub fn hash(&self) -> &str { self.inner.fragment().unwrap_or("") }
    pub fn host(&self) -> &str { self.inner.host_str().unwrap_or("") }
    pub fn origin(&self) -> String { format!("{}://{}", self.inner.scheme(), self.inner.host_str().unwrap_or("")) }
    pub fn href(&self) -> &str { self.inner.as_str() }
    pub fn search_params(&self) -> UrlSearchParams { UrlSearchParams::from_query(self.inner.query().unwrap_or("")) }
    pub fn set_protocol(&mut self, protocol: &str) { self.inner.set_scheme(protocol).ok(); }
    pub fn set_hostname(&mut self, hostname: &str) { self.inner.set_host(Some(hostname)).ok(); }
    pub fn set_port(&mut self, port: Option<u16>) { self.inner.set_port(port).ok(); }
    pub fn set_pathname(&mut self, path: &str) { self.inner.set_path(path); }
    pub fn set_search(&mut self, query: &str) { self.inner.set_query(Some(query.trim_start_matches('?'))); }
    pub fn set_hash(&mut self, hash: &str) { self.inner.set_fragment(Some(hash.trim_start_matches('#'))); }
    pub fn join(&self, relative: &str) -> anyhow::Result<Self> { Ok(Self { inner: self.inner.join(relative)? }) }
    pub fn to_string(&self) -> String { self.inner.to_string() }
}
impl std::fmt::Display for Url { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.inner) } }
impl std::str::FromStr for Url { type Err = anyhow::Error; fn from_str(s: &str) -> Result<Self, Self::Err> { Self::parse(s) } }

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
    pub fn new(method: &str, url: &str) -> Self { Self { method: method.to_string(), url: url.to_string(), headers: HashMap::new(), body: None } }
    pub fn header(&mut self, name: &str, value: &str) -> &mut Self { self.headers.insert(name.to_string(), value.to_string()); self }
    pub fn body(&mut self, body: &str) -> &mut Self { self.body = Some(body.to_string()); self }
    pub fn json<T: Serialize>(&mut self, data: &T) -> anyhow::Result<&mut Self> { self.body = Some(serde_json::to_string(data)?); self.header("Content-Type", "application/json"); Ok(self) }
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
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<T> { Ok(serde_json::from_str(&self.body)?) }
    pub fn text(&self) -> &str { &self.body }
    pub fn ok(&self) -> bool { self.status >= 200 && self.status < 300 }
    pub fn status_code(&self) -> u16 { self.status }
    pub fn header(&self, name: &str) -> Option<&str> { self.headers.get(&name.to_lowercase()).map(|s| s.as_str()) }
}
