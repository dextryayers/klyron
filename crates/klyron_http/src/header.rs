use std::collections::HashMap;

pub fn parse_headers(headers: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for line in headers.lines() {
        if let Some((key, value)) = line.split_once(':') {
            result.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    result
}

pub fn format_headers(headers: &HashMap<String, String>) -> String {
    headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<_>>()
        .join("\r\n")
}

pub fn get_content_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        "txt" => "text/plain",
        "yaml" | "yml" => "text/yaml",
        "md" => "text/markdown",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

pub fn build_cors_headers(
    origin: &str,
    methods: &[&str],
    headers: &[&str],
) -> Vec<(&'static str, String)> {
    vec![
        ("Access-Control-Allow-Origin", origin.to_string()),
        (
            "Access-Control-Allow-Methods",
            methods.join(", "),
        ),
        (
            "Access-Control-Allow-Headers",
            headers.join(", "),
        ),
    ]
}

#[derive(Debug, Clone)]
pub struct HeaderMap {
    inner: Vec<(String, String)>,
}

impl HeaderMap {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.inner.push((key.into(), value.into()));
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        for (k, v) in &self.inner {
            if k.eq_ignore_ascii_case(key) {
                return Some(v.as_str());
            }
        }
        None
    }

    pub fn all(&self, key: &str) -> Vec<&str> {
        self.inner
            .iter()
            .filter(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn remove(&mut self, key: &str) {
        self.inner.retain(|(k, _)| !k.eq_ignore_ascii_case(key));
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for HeaderMap {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<(String, String)> for HeaderMap {
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}
