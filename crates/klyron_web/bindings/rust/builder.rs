use crate::types::{Request, Url, UrlSearchParams};

pub struct RequestBuilder {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}
impl RequestBuilder {
    pub fn new(method: &str, url: &str) -> Self { Self { method: method.into(), url: url.into(), headers: Vec::new(), body: None } }
    pub fn header(mut self, name: &str, value: &str) -> Self { self.headers.push((name.into(), value.into())); self }
    pub fn body(mut self, body: &str) -> Self { self.body = Some(body.into()); self }
    pub fn build(self) -> Request { let mut req = Request::new(&self.method, &self.url); for (k,v) in self.headers { req.headers.insert(k,v); } req.body = self.body; req }
}

pub struct UrlBuilder { base: String, params: Vec<(String, String)> }
impl UrlBuilder {
    pub fn new(base: &str) -> Self { Self { base: base.into(), params: Vec::new() } }
    pub fn query(mut self, key: &str, value: &str) -> Self { self.params.push((key.into(), value.into())); self }
    pub fn build(self) -> anyhow::Result<Url> { let mut url = Url::parse(&self.base)?; if !self.params.is_empty() { let mut sp = UrlSearchParams::new(); for (k,v) in self.params { sp.append(&k, &v); } url.set_search(&sp.to_string()); } Ok(url) }
}
