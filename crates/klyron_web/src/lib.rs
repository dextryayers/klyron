pub mod fetch;
pub mod sse;
pub mod websocket;

use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::Bytes;
use futures_util::Stream;
use serde::Serialize;

pub use fetch::*;
pub use sse::*;
pub use websocket::*;

#[derive(Debug, Clone)]
pub struct Headers {
    inner: HashMap<String, Vec<String>>,
}

impl Headers {
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner
            .get(&name.to_lowercase())
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }

    pub fn get_all(&self, name: &str) -> Vec<&str> {
        self.inner
            .get(&name.to_lowercase())
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn set(&mut self, name: &str, value: &str) {
        self.inner.insert(name.to_lowercase(), vec![value.to_string()]);
    }

    pub fn append(&mut self, name: &str, value: &str) {
        self.inner
            .entry(name.to_lowercase())
            .or_default()
            .push(value.to_string());
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
        self.inner
            .iter()
            .flat_map(|(k, vals)| vals.iter().map(move |v| (k.as_str(), v.as_str())))
    }

    pub fn into_hashmap(self) -> HashMap<String, String> {
        self.inner
            .into_iter()
            .map(|(k, mut v)| (k, v.remove(0)))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, String>> for Headers {
    fn from(map: HashMap<String, String>) -> Self {
        let mut h = Headers::new();
        for (k, v) in map {
            h.set(&k, &v);
        }
        h
    }
}

pub struct HeadersIntoIter {
    inner: std::collections::hash_map::IntoIter<String, Vec<String>>,
    current_key: Option<String>,
    current_vals: std::vec::IntoIter<String>,
}

impl Iterator for HeadersIntoIter {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref _key) = self.current_key {
                if let Some(val) = self.current_vals.next() {
                    return Some((self.current_key.clone().unwrap(), val));
                }
            }
            match self.inner.next() {
                Some((key, vals)) => {
                    let mut vals_iter = vals.into_iter();
                    if let Some(val) = vals_iter.next() {
                        self.current_key = Some(key);
                        self.current_vals = vals_iter;
                        return Some((self.current_key.clone().unwrap(), val));
                    }
                }
                None => return None,
            }
        }
    }
}

impl IntoIterator for Headers {
    type Item = (String, String);
    type IntoIter = HeadersIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        HeadersIntoIter {
            inner: self.inner.into_iter(),
            current_key: None,
            current_vals: Vec::new().into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a str, &'a str);
    type IntoIter = HeadersIterRef<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HeadersIterRef {
            inner: self.inner.iter(),
        }
    }
}

pub struct HeadersIterRef<'a> {
    inner: std::collections::hash_map::Iter<'a, String, Vec<String>>,
}

impl<'a> Iterator for HeadersIterRef<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .and_then(|(k, vals)| vals.first().map(|v| (k.as_str(), v.as_str())))
    }
}

#[derive(Debug, Clone)]
pub struct UrlSearchParams {
    params: Vec<(String, String)>,
}

impl UrlSearchParams {
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    pub fn from_query(query: &str) -> Self {
        let query = query.trim_start_matches('?');
        let mut params = Vec::new();
        for pair in query.split('&').filter(|s| !s.is_empty()) {
            if let Some(idx) = pair.find('=') {
                let k = urlencoding::decode(&pair[..idx]).unwrap_or_else(|_| pair[..idx].into());
                let v = urlencoding::decode(&pair[idx + 1..])
                    .unwrap_or_else(|_| pair[idx + 1..].into());
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
        self.params
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn get_all(&self, key: &str) -> Vec<&str> {
        self.params
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .collect()
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
        self.params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    urlencoding::encode(k),
                    urlencoding::encode(v)
                )
            })
            .collect::<Vec<_>>()
            .join("&")
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }
}

impl Default for UrlSearchParams {
    fn default() -> Self {
        Self::new()
    }
}

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
        Ok(Self {
            inner: url::Url::parse(input)?,
        })
    }

    pub fn parse_with_base(input: &str, base: &str) -> anyhow::Result<Self> {
        let base_url = url::Url::parse(base)?;
        Ok(Self {
            inner: base_url.join(input)?,
        })
    }

    pub fn can_parse(input: &str) -> bool {
        url::Url::parse(input).is_ok()
    }

    pub fn protocol(&self) -> &str {
        self.inner.scheme()
    }

    pub fn hostname(&self) -> &str {
        self.inner.host_str().unwrap_or("")
    }

    pub fn port(&self) -> Option<u16> {
        self.inner.port()
    }

    pub fn pathname(&self) -> &str {
        self.inner.path()
    }

    pub fn search(&self) -> &str {
        self.inner.query().unwrap_or("")
    }

    pub fn hash(&self) -> &str {
        self.inner.fragment().unwrap_or("")
    }

    pub fn host(&self) -> &str {
        self.inner.host_str().unwrap_or("")
    }

    pub fn origin(&self) -> String {
        format!(
            "{}://{}",
            self.inner.scheme(),
            self.inner.host_str().unwrap_or("")
        )
    }

    pub fn href(&self) -> &str {
        self.inner.as_str()
    }

    pub fn search_params(&self) -> UrlSearchParams {
        UrlSearchParams::from_query(self.inner.query().unwrap_or(""))
    }

    pub fn set_protocol(&mut self, protocol: &str) {
        self.inner.set_scheme(protocol).ok();
    }

    pub fn set_hostname(&mut self, hostname: &str) {
        self.inner.set_host(Some(hostname)).ok();
    }

    pub fn set_port(&mut self, port: Option<u16>) {
        self.inner.set_port(port).ok();
    }

    pub fn set_pathname(&mut self, path: &str) {
        self.inner.set_path(path);
    }

    pub fn set_search(&mut self, query: &str) {
        self.inner
            .set_query(Some(query.trim_start_matches('?')));
    }

    pub fn set_hash(&mut self, hash: &str) {
        self.inner
            .set_fragment(Some(hash.trim_start_matches('#')));
    }

    pub fn join(&self, relative: &str) -> anyhow::Result<Self> {
        Ok(Self {
            inner: self.inner.join(relative)?,
        })
    }

    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::str::FromStr for Url {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    signal: Option<AbortSignal>,
}

impl Request {
    pub fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
            signal: None,
        }
    }

    pub fn header(&mut self, name: &str, value: &str) -> &mut Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn body(&mut self, data: Vec<u8>) -> &mut Self {
        self.body = Some(data);
        self
    }

    pub fn text_body(&mut self, data: &str) -> &mut Self {
        self.body = Some(data.as_bytes().to_vec());
        self
    }

    pub fn json<T: Serialize>(&mut self, data: &T) -> anyhow::Result<&mut Self> {
        self.body = Some(serde_json::to_vec(data)?);
        self.header("Content-Type", "application/json");
        Ok(self)
    }

    pub fn signal(&mut self, signal: AbortSignal) -> &mut Self {
        self.signal = Some(signal);
        self
    }

    pub fn clone_request(&self) -> Self {
        Self {
            method: self.method.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
            signal: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbortSignal {
    aborted: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl AbortSignal {
    pub fn new() -> Self {
        Self {
            aborted: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn timeout(dur: Duration) -> Self {
        let signal = Self::new();
        let aborted = signal.aborted.clone();
        tokio::spawn(async move {
            tokio::time::sleep(dur).await;
            aborted.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        signal
    }

    pub fn abort(&self) {
        self.aborted
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for AbortSignal {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub url: String,
}

impl Response {
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<T> {
        Ok(serde_json::from_slice(&self.body)?)
    }

    pub fn text(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }

    pub fn ok(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn status_code(&self) -> u16 {
        self.status
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_lowercase())
            .map(|s| s.as_str())
    }

    pub fn clone_response(&self) -> Self {
        Self {
            status: self.status,
            status_text: self.status_text.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
            url: self.url.clone(),
        }
    }
}

pub struct ResponseBody {
    stream: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>>,
}

impl ResponseBody {
    pub fn new(resp: reqwest::Response) -> Self {
        Self {
            stream: Box::pin(resp.bytes_stream()),
        }
    }
}

impl Stream for ResponseBody {
    type Item = reqwest::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}

#[derive(Debug, Clone)]
pub struct WebApi;

impl WebApi {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(&self, url: &str) -> anyhow::Result<Response> {
        fetch_url(url).await
    }

    pub async fn request(&self, req: &Request) -> anyhow::Result<Response> {
        crate::fetch::fetch(req).await
    }

    pub async fn get(&self, url: &str) -> anyhow::Result<Response> {
        crate::fetch::fetch(&Request::new("GET", url)).await
    }

    pub async fn post(&self, url: &str, body: &str) -> anyhow::Result<Response> {
        let mut req = Request::new("POST", url);
        req.text_body(body);
        crate::fetch::fetch(&req).await
    }

    pub async fn post_json<T: Serialize>(&self, url: &str, data: &T) -> anyhow::Result<Response> {
        let mut req = Request::new("POST", url);
        req.header("Content-Type", "application/json");
        req.body = Some(serde_json::to_vec(data)?);
        crate::fetch::fetch(&req).await
    }

    pub fn parse_url(&self, input: &str) -> anyhow::Result<Url> {
        Url::parse(input)
    }
}

impl Default for WebApi {
    fn default() -> Self {
        Self::new()
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
    fn test_headers_into_iter() {
        let mut h = Headers::new();
        h.set("a", "1");
        h.set("b", "2");
        let pairs: Vec<_> = h.into_iter().collect();
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_url_parse() {
        let url =
            Url::parse("https://user:pass@example.com:8080/path/to?query=1&key=val#frag").unwrap();
        assert_eq!(url.protocol(), "https");
        assert_eq!(url.hostname(), "example.com");
        assert_eq!(url.port(), Some(8080));
        assert_eq!(url.pathname(), "/path/to");
        assert_eq!(url.hash(), "frag");
    }

    #[test]
    fn test_url_can_parse() {
        assert!(Url::can_parse("https://example.com"));
        assert!(!Url::can_parse("not a url"));
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
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: b"{}".to_vec(),
            url: "http://example.com".to_string(),
        };
        assert!(resp.ok());
    }

    #[test]
    fn test_response_json() {
        let resp = Response {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: br#"{"name":"test"}"#.to_vec(),
            url: "".to_string(),
        };
        let val: serde_json::Value = resp.json().unwrap();
        assert_eq!(val["name"], "test");
    }
}
