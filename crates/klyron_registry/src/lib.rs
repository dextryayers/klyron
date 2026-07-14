use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha512};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};
use thiserror::Error;

// ── Constants ────────────────────────────────────────────────────────────────

const USER_AGENT_STR: &str = "klyron-registry/0.1.0";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum RegistryError {
  #[error("HTTP error: {0}")]
  HttpError(String),
  #[error("Not found: {0}")]
  NotFound(String),
  #[error("Rate limited: retry after {0:?}")]
  RateLimited(Duration),
  #[error("Authentication error: {0}")]
  AuthError(String),
  #[error("Parse error: {0}")]
  ParseError(String),
  #[error("Cache error: {0}")]
  CacheError(String),
  #[error("Unsupported registry: {0}")]
  UnsupportedRegistry(String),
}

// ── Rate Limiter ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RateLimiter {
  pub max_requests: u32,
  pub window: Duration,
  counter: u32,
  window_start: Instant,
}

impl RateLimiter {
  pub fn new(max_requests: u32, window_secs: u64) -> Self {
    Self {
      max_requests,
      window: Duration::from_secs(window_secs),
      counter: 0,
      window_start: Instant::now(),
    }
  }

  pub fn check(&mut self) -> Result<(), RegistryError> {
    let now = Instant::now();
    if now - self.window_start > self.window {
      self.counter = 0;
      self.window_start = now;
    }
    if self.counter >= self.max_requests {
      let retry_after = self.window - (now - self.window_start);
      return Err(RegistryError::RateLimited(retry_after));
    }
    self.counter += 1;
    Ok(())
  }
}

impl Default for RateLimiter {
  fn default() -> Self {
    Self::new(60, 60)
  }
}

// ── Cache ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CacheEntry {
  pub data: Value,
  pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RegistryCache {
  entries: HashMap<String, CacheEntry>,
  default_ttl: Duration,
}

impl RegistryCache {
  pub fn new(ttl_secs: u64) -> Self {
    Self {
      entries: HashMap::new(),
      default_ttl: Duration::from_secs(ttl_secs),
    }
  }

  pub fn get(&self, key: &str) -> Option<Value> {
    self.entries.get(key).and_then(|entry| {
      if Utc::now() < entry.expires_at {
        Some(entry.data.clone())
      } else {
        None
      }
    })
  }

  pub fn set(&mut self, key: String, value: Value, ttl: Option<Duration>) {
    let ttl = ttl.unwrap_or(self.default_ttl);
    self.entries.insert(
      key,
      CacheEntry {
        data: value,
        expires_at: Utc::now() + chrono::Duration::from_std(ttl).unwrap_or_default(),
      },
    );
  }

  pub fn invalidate(&mut self, key: &str) {
    self.entries.remove(key);
  }

  pub fn clear(&mut self) {
    self.entries.clear();
  }

  pub fn len(&self) -> usize {
    self.entries.len()
  }

  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }
}

impl Default for RegistryCache {
  fn default() -> Self {
    Self::new(300)
  }
}

// ── Package Info ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
  pub name: String,
  pub version: String,
  pub description: Option<String>,
  pub license: Option<String>,
  pub homepage: Option<String>,
  pub repository: Option<String>,
  pub author: Option<String>,
  pub keywords: Vec<String>,
  pub registry: RegistryKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSearchResult {
  pub results: Vec<PackageInfo>,
  pub total: usize,
  pub took_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDownload {
  pub name: String,
  pub version: String,
  pub data: Vec<u8>,
  pub integrity: String,
  pub content_type: String,
}

// ── Registry Kind ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegistryKind {
  Npm,
  PyPI,
  RubyGems,
  Cargo,
  Packagist,
  JSR,
  Deno,
}

impl RegistryKind {
  pub fn detect(name: &str) -> Self {
    // PHP packages have vendor/package format
    if name.contains('/') && !name.starts_with('@') {
      return Self::Packagist;
    }
    // npm scoped packages
    if name.starts_with('@') {
      return Self::Npm;
    }
    // Python packages often use underscores
    if name.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c == '-')
      && name.contains('_')
    {
      return Self::PyPI;
    }
    // Ruby gems: typically single-word with underscore, or prefixed with 'ruby-', suffixed with '_gem'
    if (name.starts_with("ruby-") || name.ends_with("_gem"))
      && name.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c == '-')
    {
      return Self::RubyGems;
    }
    // Default to npm for everything else
    Self::Npm
  }

  pub fn name(&self) -> &str {
    match self {
      Self::Npm => "npm",
      Self::PyPI => "pypi",
      Self::RubyGems => "rubygems",
      Self::Cargo => "cargo",
      Self::Packagist => "packagist",
      Self::JSR => "jsr",
      Self::Deno => "deno",
    }
  }
}

impl fmt::Display for RegistryKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}

// ── Registry ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RegistryConfig {
  pub registry_url: String,
  pub mirrors: Vec<String>,
  pub auth_token: Option<String>,
  pub timeout: Duration,
  pub cache_ttl: Duration,
  pub rate_limit: RateLimiter,
}

impl Default for RegistryConfig {
  fn default() -> Self {
    Self {
      registry_url: "https://registry.npmjs.org".into(),
      mirrors: vec![
        "https://registry.npmmirror.com".into(),
        "https://unpkg.com".into(),
      ],
      auth_token: None,
      timeout: DEFAULT_TIMEOUT,
      cache_ttl: Duration::from_secs(300),
      rate_limit: RateLimiter::new(60, 60),
    }
  }
}

// ── NpmRegistry ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NpmRegistry {
  pub config: RegistryConfig,
  cache: RefCell<RegistryCache>,
}

impl NpmRegistry {
  pub fn new() -> Self {
    Self {
      config: RegistryConfig::default(),
      cache: RefCell::new(RegistryCache::new(300)),
    }
  }

  pub fn with_config(config: RegistryConfig) -> Self {
    Self {
      cache: RefCell::new(RegistryCache::new(config.cache_ttl.as_secs())),
      config,
    }
  }

  pub fn search(&self, query: &str, limit: usize) -> Result<PackageSearchResult, RegistryError> {
    let url = format!(
      "{}/-/v1/search?text={}&size={}",
      self.config.registry_url,
      urlencoding(query),
      limit,
    );
    let resp = self.http_get(&url)?;
    let packages = resp.get("objects").and_then(|o| o.as_array()).map(|arr| {
      arr
        .iter()
        .filter_map(|obj| {
          let pkg = obj.get("package")?;
          Some(PackageInfo {
            name: pkg.get("name")?.as_str()?.to_string(),
            version: pkg.get("version")?.as_str()?.to_string(),
            description: pkg.get("description").and_then(|v| v.as_str()).map(String::from),
            license: pkg.get("license").and_then(|v| v.as_str()).map(String::from),
            homepage: pkg.get("links").and_then(|l| l.get("homepage")).and_then(|v| v.as_str()).map(String::from),
            repository: pkg.get("links").and_then(|l| l.get("repository")).and_then(|v| v.as_str()).map(String::from),
            author: None,
            keywords: pkg.get("keywords").and_then(|k| k.as_array()).map(|a| {
              a.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            }).unwrap_or_default(),
            registry: RegistryKind::Npm,
          })
        })
        .collect::<Vec<_>>()
    }).unwrap_or_default();

    let total = resp.get("total").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

    Ok(PackageSearchResult {
      results: packages,
      total,
      took_ms: 0,
    })
  }

  pub fn info(&self, name: &str) -> Result<PackageInfo, RegistryError> {
    let cache_key = format!("npm:info:{name}");
    if let Some(cached) = self.cache.borrow().get(&cache_key) {
      return serde_json::from_value(cached)
        .map_err(|e| RegistryError::ParseError(e.to_string()));
    }

    let url = if name.starts_with('@') {
      let encoded = name.replace('/', "%2F");
      format!("{}/{}", self.config.registry_url, encoded)
    } else {
      format!("{}/{}", self.config.registry_url, name)
    };

    let resp = self.http_get(&url)?;
    let latest = resp.get("dist-tags").and_then(|t| t.get("latest")).and_then(|v| v.as_str()).unwrap_or("unknown");
    let pkg_info = PackageInfo {
      name: name.to_string(),
      version: latest.to_string(),
      description: resp.get("description").and_then(|v| v.as_str()).map(String::from),
      license: resp.get("license").and_then(|v| v.as_str()).map(String::from),
      homepage: resp.get("homepage").and_then(|v| v.as_str()).map(String::from),
      repository: resp.get("repository").and_then(|r| r.as_str().or_else(|| r.get("url").and_then(|u| u.as_str()))).map(String::from),
      author: resp.get("author").and_then(|a| a.as_str().or_else(|| a.get("name").and_then(|n| n.as_str()))).map(String::from),
      keywords: resp.get("keywords").and_then(|k| k.as_array()).map(|a| {
        a.iter().filter_map(|v| v.as_str().map(String::from)).collect()
      }).unwrap_or_default(),
      registry: RegistryKind::Npm,
    };

    if let Ok(val) = serde_json::to_value(&pkg_info) {
      self.cache.borrow_mut().set(cache_key, val, None);
    }

    Ok(pkg_info)
  }

  pub fn download(&self, name: &str, version: &str) -> Result<PackageDownload, RegistryError> {
    let url = format!("{}/{}/-/{}-{}.tgz", self.config.registry_url, name, name.replace('/', "-"), version);
    let tarball_key = format!("npm:tarball:{name}@{version}");
    if let Some(cached) = self.cache.borrow().get(&tarball_key) {
      if let Some(hex_str) = cached.as_str() {
        if let Ok(data) = hex::decode(hex_str) {
          let integrity = compute_integrity(&data);
          return Ok(PackageDownload {
            name: name.to_string(),
            version: version.to_string(),
            data,
            integrity,
            content_type: "application/octet-stream".into(),
          });
        }
      }
    }

    let client = self.make_client()?;
    let response = client
      .get(&url)
      .send()
      .map_err(|e| RegistryError::HttpError(format!("Download failed: {e}")))?;

    if !response.status().is_success() {
      return Err(RegistryError::NotFound(format!(
        "Failed to download {name}@{version}: HTTP {}",
        response.status()
      )));
    }

    let data = response
      .bytes()
      .map_err(|e| RegistryError::HttpError(format!("Read failed: {e}")))?
      .to_vec();

    let integrity = compute_integrity(&data);
    let hex_str = hex::encode(&data);
    self.cache.borrow_mut().set(
      tarball_key,
      Value::String(hex_str),
      Some(Duration::from_secs(3600)),
    );

    Ok(PackageDownload {
      name: name.to_string(),
      version: version.to_string(),
      data,
      integrity,
      content_type: "application/gzip".into(),
    })
  }

  pub fn login(&mut self, token: &str) -> Result<(), RegistryError> {
    self.config.auth_token = Some(token.to_string());
    Ok(())
  }

  pub fn logout(&mut self) {
    self.config.auth_token = None;
  }

  fn http_get(&self, url: &str) -> Result<Value, RegistryError> {
    let client = self.make_client()?;
    let response = client
      .get(url)
      .send()
      .map_err(|e| RegistryError::HttpError(format!("GET {url} failed: {e}")))?;

    if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
      let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(60));
      return Err(RegistryError::RateLimited(retry_after));
    }

    if !response.status().is_success() {
      return Err(RegistryError::HttpError(format!(
        "HTTP {} for {url}",
        response.status()
      )));
    }

    response
      .json::<Value>()
      .map_err(|e| RegistryError::ParseError(format!("JSON parse error: {e}")))
  }

  fn make_client(&self) -> Result<Client, RegistryError> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    let mut builder = Client::builder()
      .timeout(self.config.timeout)
      .default_headers(headers);

    if let Some(ref token) = self.config.auth_token {
      let mut auth_headers = HeaderMap::new();
      auth_headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {token}"))
          .map_err(|_| RegistryError::AuthError("Invalid auth token".into()))?,
      );
      builder = builder.default_headers(auth_headers);
    }

    builder
      .build()
      .map_err(|e| RegistryError::HttpError(format!("Client build error: {e}")))
  }
}

impl Default for NpmRegistry {
  fn default() -> Self {
    Self::new()
  }
}

// ── PyPIRegistry ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PyPIRegistry {
  pub config: RegistryConfig,
  cache: RefCell<RegistryCache>,
}

impl PyPIRegistry {
  pub fn new() -> Self {
    let mut config = RegistryConfig::default();
    config.registry_url = "https://pypi.org/pypi".into();
    config.mirrors = vec![
      "https://mirrors.aliyun.com/pypi/".into(),
      "https://pypi.douban.com/simple/".into(),
    ];
    Self {
      config,
      cache: RefCell::new(RegistryCache::new(300)),
    }
  }

  pub fn search(&self, query: &str, limit: usize) -> Result<PackageSearchResult, RegistryError> {
    let url = format!(
      "{}/search/?q={}&per_page={}",
      self.config.registry_url,
      urlencoding(query),
      limit,
    );
    let resp = self.http_get(&url)?;
    let results = resp
      .get("results")
      .and_then(|r| r.as_array())
      .map(|arr| {
        arr
          .iter()
          .filter_map(|item| {
            Some(PackageInfo {
              name: item.get("name")?.as_str()?.to_string(),
              version: item.get("version")?.as_str()?.to_string(),
              description: item.get("summary").and_then(|v| v.as_str()).map(String::from),
              license: item.get("license").and_then(|v| v.as_str()).map(String::from),
              homepage: item.get("home_page").and_then(|v| v.as_str()).map(String::from),
              repository: item.get("project_urls").and_then(|u| u.as_object())
                .and_then(|m| m.get("Source")).and_then(|v| v.as_str()).map(String::from),
              author: item.get("author").and_then(|v| v.as_str()).map(String::from),
              keywords: item.get("keywords").and_then(|v| v.as_str())
                .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
                .unwrap_or_default(),
              registry: RegistryKind::PyPI,
            })
          })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    let total = results.len();
    Ok(PackageSearchResult {
      results,
      total,
      took_ms: 0,
    })
  }

  pub fn info(&self, name: &str) -> Result<PackageInfo, RegistryError> {
    let cache_key = format!("pypi:info:{name}");
    if let Some(cached) = self.cache.borrow().get(&cache_key) {
      return serde_json::from_value(cached).map_err(|e| RegistryError::ParseError(e.to_string()));
    }

    let url = format!("{}/{}/json", self.config.registry_url, name);
    let resp = self.http_get(&url)?;
    let info = resp.get("info").ok_or_else(|| RegistryError::NotFound(name.into()))?;

    let pkg_info = PackageInfo {
      name: info.get("name").and_then(|v| v.as_str()).unwrap_or(name).to_string(),
      version: info.get("version").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
      description: info.get("summary").and_then(|v| v.as_str()).map(String::from),
      license: info.get("license").and_then(|v| v.as_str()).map(String::from),
      homepage: info.get("home_page").and_then(|v| v.as_str()).map(String::from),
      repository: info.get("project_urls").and_then(|u| u.as_object())
        .and_then(|m| m.get("Source")).and_then(|v| v.as_str()).map(String::from),
      author: info.get("author").and_then(|v| v.as_str()).map(String::from),
      keywords: info.get("keywords").and_then(|v| v.as_str())
        .map(|s| s.split(',').map(|k| k.trim().to_string()).collect())
        .unwrap_or_default(),
      registry: RegistryKind::PyPI,
    };

    if let Ok(val) = serde_json::to_value(&pkg_info) {
      self.cache.borrow_mut().set(cache_key, val, None);
    }

    Ok(pkg_info)
  }

  pub fn download(&self, name: &str, version: &str) -> Result<PackageDownload, RegistryError> {
    let url = format!(
      "{}/packages/{}/{}-{}.tar.gz",
      self.config.registry_url.trim_end_matches("/pypi"),
      name,
      name,
      version,
    );
    self.download_url(name, version, &url)
  }

  fn download_url(&self, name: &str, version: &str, url: &str) -> Result<PackageDownload, RegistryError> {
    let client = self.make_client()?;
    let response = client
      .get(url)
      .send()
      .map_err(|e| RegistryError::HttpError(format!("Download failed: {e}")))?;

    if !response.status().is_success() {
      return Err(RegistryError::NotFound(format!("Download failed: HTTP {}", response.status())));
    }

    let data = response.bytes().map_err(|e| RegistryError::HttpError(e.to_string()))?.to_vec();
    let integrity = compute_integrity(&data);

    Ok(PackageDownload {
      name: name.to_string(),
      version: version.to_string(),
      data,
      integrity,
      content_type: "application/gzip".into(),
    })
  }

  fn http_get(&self, url: &str) -> Result<Value, RegistryError> {
    let client = self.make_client()?;
    let response = client
      .get(url)
      .send()
      .map_err(|e| RegistryError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
      return Err(RegistryError::HttpError(format!("HTTP {} for {url}", response.status())));
    }

    response.json::<Value>().map_err(|e| RegistryError::ParseError(e.to_string()))
  }

  fn make_client(&self) -> Result<Client, RegistryError> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    Client::builder()
      .timeout(self.config.timeout)
      .default_headers(headers)
      .build()
      .map_err(|e| RegistryError::HttpError(e.to_string()))
  }
}

impl Default for PyPIRegistry {
  fn default() -> Self {
    Self::new()
  }
}

// ── RubyGemsRegistry ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RubyGemsRegistry {
  pub config: RegistryConfig,
  cache: RefCell<RegistryCache>,
}

impl RubyGemsRegistry {
  pub fn new() -> Self {
    let mut config = RegistryConfig::default();
    config.registry_url = "https://rubygems.org".into();
    Self {
      config,
      cache: RefCell::new(RegistryCache::new(300)),
    }
  }

  pub fn search(&self, query: &str, limit: usize) -> Result<PackageSearchResult, RegistryError> {
    let url = format!(
      "{}/api/v1/search.json?query={}&per_page={}",
      self.config.registry_url,
      urlencoding(query),
      limit,
    );
    let resp = self.http_get(&url)?;
    let results = resp
      .as_array()
      .map(|arr| {
        arr
          .iter()
          .filter_map(|item| {
            Some(PackageInfo {
              name: item.get("name")?.as_str()?.to_string(),
              version: item.get("version")?.as_str()?.to_string(),
              description: item.get("info").and_then(|v| v.as_str()).map(String::from),
              license: None,
              homepage: item.get("homepage_uri").and_then(|v| v.as_str()).map(String::from),
              repository: item.get("source_code_uri").and_then(|v| v.as_str()).map(String::from),
              author: item.get("authors").and_then(|v| v.as_str()).map(String::from),
              keywords: Vec::new(),
              registry: RegistryKind::RubyGems,
            })
          })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    Ok(PackageSearchResult {
      total: results.len(),
      results,
      took_ms: 0,
    })
  }

  pub fn info(&self, name: &str) -> Result<PackageInfo, RegistryError> {
    let cache_key = format!("rubygems:info:{name}");
    if let Some(cached) = self.cache.borrow().get(&cache_key) {
      return serde_json::from_value(cached).map_err(|e| RegistryError::ParseError(e.to_string()));
    }

    let url = format!("{}/api/v1/gems/{}.json", self.config.registry_url, name);
    let resp = self.http_get(&url)?;

    let pkg_info = PackageInfo {
      name: resp.get("name").and_then(|v| v.as_str()).unwrap_or(name).to_string(),
      version: resp.get("version").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
      description: resp.get("info").and_then(|v| v.as_str()).map(String::from),
      license: resp.get("license").and_then(|v| v.as_str().map(String::from)).or_else(|| {
        resp.get("licenses").and_then(|l| l.as_array()).and_then(|a| a.first()).and_then(|v| v.as_str().map(String::from))
      }),
      homepage: resp.get("homepage_uri").and_then(|v| v.as_str()).map(String::from),
      repository: resp.get("source_code_uri").and_then(|v| v.as_str()).map(String::from),
      author: resp.get("authors").and_then(|v| v.as_str()).map(String::from),
      keywords: Vec::new(),
      registry: RegistryKind::RubyGems,
    };

    if let Ok(val) = serde_json::to_value(&pkg_info) {
      self.cache.borrow_mut().set(cache_key, val, None);
    }

    Ok(pkg_info)
  }

  pub fn download(&self, name: &str, version: &str) -> Result<PackageDownload, RegistryError> {
    let url = format!(
      "{}/gems/{}-{}.gem",
      self.config.registry_url,
      name,
      version,
    );
    let client = self.make_client()?;
    let response = client
      .get(&url)
      .send()
      .map_err(|e| RegistryError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
      return Err(RegistryError::NotFound(format!("Failed to download {name}@{version}")));
    }

    let data = response.bytes().map_err(|e| RegistryError::HttpError(e.to_string()))?.to_vec();
    let integrity = compute_integrity(&data);

    Ok(PackageDownload {
      name: name.to_string(),
      version: version.to_string(),
      data,
      integrity,
      content_type: "application/octet-stream".into(),
    })
  }

  fn http_get(&self, url: &str) -> Result<Value, RegistryError> {
    let client = self.make_client()?;
    let response = client
      .get(url)
      .send()
      .map_err(|e| RegistryError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
      return Err(RegistryError::HttpError(format!("HTTP {} for {url}", response.status())));
    }

    response.json::<Value>().map_err(|e| RegistryError::ParseError(e.to_string()))
  }

  fn make_client(&self) -> Result<Client, RegistryError> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    Client::builder()
      .timeout(self.config.timeout)
      .default_headers(headers)
      .build()
      .map_err(|e| RegistryError::HttpError(e.to_string()))
  }
}

impl Default for RubyGemsRegistry {
  fn default() -> Self {
    Self::new()
  }
}

// ── CargoRegistry ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CargoRegistry {
  pub config: RegistryConfig,
  cache: RefCell<RegistryCache>,
}

impl CargoRegistry {
  pub fn new() -> Self {
    let mut config = RegistryConfig::default();
    config.registry_url = "https://crates.io/api/v1".into();
    Self {
      config,
      cache: RefCell::new(RegistryCache::new(300)),
    }
  }

  pub fn search(&self, query: &str, limit: usize) -> Result<PackageSearchResult, RegistryError> {
    let url = format!(
      "{}/crates?q={}&per_page={}",
      self.config.registry_url,
      urlencoding(query),
      limit.min(100),
    );
    let resp = self.http_get(&url)?;
    let crates = resp.get("crates").and_then(|c| c.as_array()).map(|arr| {
      arr
        .iter()
        .filter_map(|item| {
          Some(PackageInfo {
            name: item.get("id")?.as_str()?.to_string(),
            version: item.get("max_version")?.as_str()?.to_string(),
            description: item.get("description").and_then(|v| v.as_str()).map(String::from),
            license: item.get("license").and_then(|v| v.as_str()).map(String::from),
            homepage: item.get("homepage").and_then(|v| v.as_str()).map(String::from),
            repository: item.get("repository").and_then(|v| v.as_str()).map(String::from),
            author: None,
            keywords: item.get("keywords").and_then(|k| k.as_array()).map(|a| {
              a.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            }).unwrap_or_default(),
            registry: RegistryKind::Cargo,
          })
        })
        .collect::<Vec<_>>()
    }).unwrap_or_default();

    Ok(PackageSearchResult {
      total: crates.len(),
      results: crates,
      took_ms: 0,
    })
  }

  pub fn info(&self, name: &str) -> Result<PackageInfo, RegistryError> {
    let cache_key = format!("cargo:info:{name}");
    if let Some(cached) = self.cache.borrow().get(&cache_key) {
      return serde_json::from_value(cached).map_err(|e| RegistryError::ParseError(e.to_string()));
    }

    let url = format!("{}/crates/{}", self.config.registry_url, name);
    let resp = self.http_get(&url)?;
    let crate_data = resp.get("crate").ok_or_else(|| RegistryError::NotFound(name.into()))?;

    let pkg_info = PackageInfo {
      name: crate_data.get("id").and_then(|v| v.as_str()).unwrap_or(name).to_string(),
      version: crate_data.get("max_version").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
      description: crate_data.get("description").and_then(|v| v.as_str()).map(String::from),
      license: crate_data.get("license").and_then(|v| v.as_str()).map(String::from),
      homepage: crate_data.get("homepage").and_then(|v| v.as_str()).map(String::from),
      repository: crate_data.get("repository").and_then(|v| v.as_str()).map(String::from),
      author: None,
      keywords: crate_data.get("keywords").and_then(|k| k.as_array()).map(|a| {
        a.iter().filter_map(|v| v.as_str().map(String::from)).collect()
      }).unwrap_or_default(),
      registry: RegistryKind::Cargo,
    };

    if let Ok(val) = serde_json::to_value(&pkg_info) {
      self.cache.borrow_mut().set(cache_key, val, None);
    }

    Ok(pkg_info)
  }

  pub fn download(&self, name: &str, version: &str) -> Result<PackageDownload, RegistryError> {
    let url = format!(
      "https://static.crates.io/crates/{name}/{name}-{version}.crate",
    );
    let client = self.make_client()?;
    let response = client
      .get(&url)
      .send()
      .map_err(|e| RegistryError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
      return Err(RegistryError::NotFound(format!("Failed to download {name}@{version}")));
    }

    let data = response.bytes().map_err(|e| RegistryError::HttpError(e.to_string()))?.to_vec();
    let integrity = compute_integrity(&data);

    Ok(PackageDownload {
      name: name.to_string(),
      version: version.to_string(),
      data,
      integrity,
      content_type: "application/gzip".into(),
    })
  }

  fn http_get(&self, url: &str) -> Result<Value, RegistryError> {
    let client = self.make_client()?;
    let response = client
      .get(url)
      .send()
      .map_err(|e| RegistryError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
      return Err(RegistryError::HttpError(format!("HTTP {} for {url}", response.status())));
    }

    response.json::<Value>().map_err(|e| RegistryError::ParseError(e.to_string()))
  }

  fn make_client(&self) -> Result<Client, RegistryError> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    Client::builder()
      .timeout(self.config.timeout)
      .default_headers(headers)
      .build()
      .map_err(|e| RegistryError::HttpError(e.to_string()))
  }
}

impl Default for CargoRegistry {
  fn default() -> Self {
    Self::new()
  }
}

// ── PackagistRegistry ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PackagistRegistry {
  pub config: RegistryConfig,
  cache: RefCell<RegistryCache>,
}

impl PackagistRegistry {
  pub fn new() -> Self {
    let mut config = RegistryConfig::default();
    config.registry_url = "https://packagist.org".into();
    Self {
      config,
      cache: RefCell::new(RegistryCache::new(300)),
    }
  }

  pub fn search(&self, query: &str, _limit: usize) -> Result<PackageSearchResult, RegistryError> {
    let url = format!(
      "{}/search.json?q={}",
      self.config.registry_url,
      urlencoding(query),
    );
    let resp = self.http_get(&url)?;
    let results = resp
      .get("results")
      .and_then(|r| r.as_array())
      .map(|arr| {
        arr
          .iter()
          .filter_map(|item| {
            Some(PackageInfo {
              name: item.get("name")?.as_str()?.to_string(),
              version: item.get("version")?.as_str()?.to_string(),
              description: item.get("description").and_then(|v| v.as_str()).map(String::from),
              license: None,
              homepage: None,
              repository: item.get("repository").and_then(|v| v.as_str()).map(String::from),
              author: None,
              keywords: Vec::new(),
              registry: RegistryKind::Packagist,
            })
          })
          .collect::<Vec<_>>()
      })
      .unwrap_or_default();

    Ok(PackageSearchResult {
      total: results.len(),
      results,
      took_ms: 0,
    })
  }

  pub fn info(&self, name: &str) -> Result<PackageInfo, RegistryError> {
    let cache_key = format!("packagist:info:{name}");
    if let Some(cached) = self.cache.borrow().get(&cache_key) {
      return serde_json::from_value(cached).map_err(|e| RegistryError::ParseError(e.to_string()));
    }

    let url = format!("{}/p2/{}.json", self.config.registry_url, name);
    let resp = self.http_get(&url)?;
    let packages = resp.get("packages").and_then(|p| p.as_object()).ok_or_else(|| RegistryError::NotFound(name.into()))?;
    let pkg_data = packages.get(name).and_then(|p| p.as_array()).and_then(|arr| arr.first()).ok_or_else(|| RegistryError::NotFound(name.into()))?;

    let pkg_info = PackageInfo {
      name: name.to_string(),
      version: pkg_data.get("version").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
      description: pkg_data.get("description").and_then(|v| v.as_str()).map(String::from),
      license: pkg_data.get("license").and_then(|l| l.as_array()).and_then(|a| a.first()).and_then(|v| v.as_str()).map(String::from),
      homepage: None,
      repository: pkg_data.get("source").and_then(|s| s.get("url")).and_then(|v| v.as_str()).map(String::from),
      author: None,
      keywords: Vec::new(),
      registry: RegistryKind::Packagist,
    };

    if let Ok(val) = serde_json::to_value(&pkg_info) {
      self.cache.borrow_mut().set(cache_key, val, None);
    }

    Ok(pkg_info)
  }

  pub fn download(&self, name: &str, version: &str) -> Result<PackageDownload, RegistryError> {
    let url = format!(
      "https://github.com/{}/archive/{}.zip",
      name,
      version,
    );
    let client = self.make_client()?;
    let response = client
      .get(&url)
      .send()
      .map_err(|e| RegistryError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
      return Err(RegistryError::NotFound(format!("Failed to download {name}@{version}")));
    }

    let data = response.bytes().map_err(|e| RegistryError::HttpError(e.to_string()))?.to_vec();
    let integrity = compute_integrity(&data);

    Ok(PackageDownload {
      name: name.to_string(),
      version: version.to_string(),
      data,
      integrity,
      content_type: "application/zip".into(),
    })
  }

  fn http_get(&self, url: &str) -> Result<Value, RegistryError> {
    let client = self.make_client()?;
    let response = client
      .get(url)
      .send()
      .map_err(|e| RegistryError::HttpError(e.to_string()))?;

    if !response.status().is_success() {
      return Err(RegistryError::HttpError(format!("HTTP {} for {url}", response.status())));
    }

    response.json::<Value>().map_err(|e| RegistryError::ParseError(e.to_string()))
  }

  fn make_client(&self) -> Result<Client, RegistryError> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));

    Client::builder()
      .timeout(self.config.timeout)
      .default_headers(headers)
      .build()
      .map_err(|e| RegistryError::HttpError(e.to_string()))
  }
}

impl Default for PackagistRegistry {
  fn default() -> Self {
    Self::new()
  }
}

// ── Registry Auto-Detection ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum RegistryClient {
  Npm(NpmRegistry),
  PyPI(PyPIRegistry),
  RubyGems(RubyGemsRegistry),
  Cargo(CargoRegistry),
  Packagist(PackagistRegistry),
}

impl RegistryClient {
  pub fn detect(name: &str) -> Self {
    match RegistryKind::detect(name) {
      RegistryKind::Npm => Self::Npm(NpmRegistry::new()),
      RegistryKind::PyPI => Self::PyPI(PyPIRegistry::new()),
      RegistryKind::RubyGems => Self::RubyGems(RubyGemsRegistry::new()),
      RegistryKind::Cargo => Self::Cargo(CargoRegistry::new()),
      RegistryKind::Packagist => Self::Packagist(PackagistRegistry::new()),
      _ => Self::Npm(NpmRegistry::new()),
    }
  }

  pub fn search(&self, query: &str, limit: usize) -> Result<PackageSearchResult, RegistryError> {
    match self {
      Self::Npm(r) => r.search(query, limit),
      Self::PyPI(r) => r.search(query, limit),
      Self::RubyGems(r) => r.search(query, limit),
      Self::Cargo(r) => r.search(query, limit),
      Self::Packagist(r) => r.search(query, limit),
    }
  }

  pub fn info(&self, name: &str) -> Result<PackageInfo, RegistryError> {
    match self {
      Self::Npm(r) => r.info(name),
      Self::PyPI(r) => r.info(name),
      Self::RubyGems(r) => r.info(name),
      Self::Cargo(r) => r.info(name),
      Self::Packagist(r) => r.info(name),
    }
  }

  pub fn download(&self, name: &str, version: &str) -> Result<PackageDownload, RegistryError> {
    match self {
      Self::Npm(r) => r.download(name, version),
      Self::PyPI(r) => r.download(name, version),
      Self::RubyGems(r) => r.download(name, version),
      Self::Cargo(r) => r.download(name, version),
      Self::Packagist(r) => r.download(name, version),
    }
  }

  pub fn kind(&self) -> RegistryKind {
    match self {
      Self::Npm(_) => RegistryKind::Npm,
      Self::PyPI(_) => RegistryKind::PyPI,
      Self::RubyGems(_) => RegistryKind::RubyGems,
      Self::Cargo(_) => RegistryKind::Cargo,
      Self::Packagist(_) => RegistryKind::Packagist,
    }
  }
}

// ── Utility Functions ────────────────────────────────────────────────────────

fn urlencoding(input: &str) -> String {
  urlencoding_impl(input)
}

fn urlencoding_impl(input: &str) -> String {
  let mut result = String::with_capacity(input.len());
  for byte in input.bytes() {
    match byte {
      b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
        result.push(byte as char);
      }
      b' ' => result.push_str("%20"),
      _ => {
        result.push_str(&format!("%{:02X}", byte));
      }
    }
  }
  result
}

pub fn compute_integrity(data: &[u8]) -> String {
  let mut hasher = Sha512::new();
  hasher.update(data);
  format!("sha512-{}", hex::encode(hasher.finalize()))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_registry_kind_detect_npm() {
    assert_eq!(RegistryKind::detect("@angular/core"), RegistryKind::Npm);
    assert_eq!(RegistryKind::detect("express"), RegistryKind::Npm);
    assert_eq!(RegistryKind::detect("react"), RegistryKind::Npm);
  }

  #[test]
  fn test_registry_kind_detect_pypi() {
    assert_eq!(RegistryKind::detect("numpy"), RegistryKind::Npm);
    assert_eq!(RegistryKind::detect("py_openssl"), RegistryKind::PyPI);
  }

  #[test]
  fn test_registry_kind_detect_packagist() {
    assert_eq!(RegistryKind::detect("vendor/package"), RegistryKind::Packagist);
  }

  #[test]
  fn test_registry_kind_name() {
    assert_eq!(RegistryKind::Npm.name(), "npm");
    assert_eq!(RegistryKind::PyPI.name(), "pypi");
    assert_eq!(RegistryKind::Cargo.name(), "cargo");
    assert_eq!(RegistryKind::RubyGems.name(), "rubygems");
    assert_eq!(RegistryKind::Packagist.name(), "packagist");
  }

  #[test]
  fn test_rate_limiter() {
    let mut limiter = RateLimiter::new(5, 60);
    for _ in 0..5 {
      assert!(limiter.check().is_ok());
    }
    assert!(limiter.check().is_err());
  }

  #[test]
  fn test_cache_ttl() {
    let mut cache = RegistryCache::new(1);
    cache.set("key".into(), json!("value"), None);
    assert_eq!(cache.get("key"), Some(json!("value")));
    std::thread::sleep(Duration::from_millis(1100));
    assert_eq!(cache.get("key"), None);
  }

  #[test]
  fn test_cache_invalidate() {
    let mut cache = RegistryCache::new(60);
    cache.set("key".into(), json!("value"), None);
    assert!(cache.get("key").is_some());
    cache.invalidate("key");
    assert!(cache.get("key").is_none());
  }

  #[test]
  fn test_cache_clear() {
    let mut cache = RegistryCache::new(60);
    cache.set("a".into(), json!(1), None);
    cache.set("b".into(), json!(2), None);
    assert_eq!(cache.len(), 2);
    cache.clear();
    assert!(cache.is_empty());
  }

  #[test]
  fn test_compute_integrity() {
    let hash = compute_integrity(b"test");
    assert!(hash.starts_with("sha512-"));
    assert_eq!(hash.len(), 128 + 7); // sha512- + hex
  }

  #[test]
  fn test_urlencoding() {
    assert_eq!(urlencoding("hello world"), "hello%20world");
    assert_eq!(urlencoding("foo/bar"), "foo%2Fbar");
    assert_eq!(urlencoding("simple"), "simple");
  }

  #[test]
  fn test_registry_client_detect() {
    let client = RegistryClient::detect("express");
    assert_eq!(client.kind(), RegistryKind::Npm);
    let client = RegistryClient::detect("vendor/php-pkg");
    assert_eq!(client.kind(), RegistryKind::Packagist);
  }

  #[test]
  fn test_package_info_serde() {
    let info = PackageInfo {
      name: "test".into(),
      version: "1.0.0".into(),
      description: Some("A test package".into()),
      license: Some("MIT".into()),
      homepage: Some("https://example.com".into()),
      repository: Some("https://github.com/test/test".into()),
      author: Some("Test Author".into()),
      keywords: vec!["test".into()],
      registry: RegistryKind::Npm,
    };
    let json = serde_json::to_value(&info).unwrap();
    let deserialized: PackageInfo = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.name, "test");
    assert_eq!(deserialized.registry, RegistryKind::Npm);
  }

  #[test]
  fn test_package_search_result() {
    let result = PackageSearchResult {
      results: Vec::new(),
      total: 0,
      took_ms: 0,
    };
    assert!(result.results.is_empty());
    assert_eq!(result.total, 0);
  }

  #[test]
  fn test_package_download() {
    let download = PackageDownload {
      name: "test".into(),
      version: "1.0.0".into(),
      data: vec![1, 2, 3],
      integrity: "sha512-test".into(),
      content_type: "application/octet-stream".into(),
    };
    assert_eq!(download.name, "test");
    assert_eq!(download.data.len(), 3);
  }

  #[test]
  fn test_npm_registry_invalid_name() {
    let registry = NpmRegistry::new();
    // Should not crash with invalid names
    let _result = registry.info("this-package-definitely-does-not-exist-xyz-12345");
    // This may or may not fail depending on network
    // We just test it doesn't panic
  }

  #[test]
  fn test_pypi_registry_new() {
    let registry = PyPIRegistry::new();
    assert!(registry.config.registry_url.contains("pypi"));
  }

  #[test]
  fn test_rubygems_registry_new() {
    let registry = RubyGemsRegistry::new();
    assert!(registry.config.registry_url.contains("rubygems"));
  }

  #[test]
  fn test_cargo_registry_new() {
    let registry = CargoRegistry::new();
    assert!(registry.config.registry_url.contains("crates.io"));
  }

  #[test]
  fn test_packagist_registry_new() {
    let registry = PackagistRegistry::new();
    assert!(registry.config.registry_url.contains("packagist"));
  }

  #[test]
  fn test_registry_config_default() {
    let config = RegistryConfig::default();
    assert_eq!(config.mirrors.len(), 2);
    assert!(config.auth_token.is_none());
  }

  #[test]
  fn test_registry_error_types() {
    let e1 = RegistryError::NotFound("test".into());
    let e2 = RegistryError::RateLimited(Duration::from_secs(30));
    let e3 = RegistryError::HttpError("connection failed".into());
    assert!(e1.to_string().contains("test"));
    assert!(e2.to_string().contains("30"));
    assert!(e3.to_string().contains("connection failed"));
  }

  #[test]
  fn test_registry_kind_display() {
    assert_eq!(format!("{}", RegistryKind::Npm), "npm");
    assert_eq!(format!("{}", RegistryKind::PyPI), "pypi");
  }

  #[test]
  fn test_npm_login_logout() {
    let mut registry = NpmRegistry::new();
    assert!(registry.config.auth_token.is_none());
    registry.login("test-token-123").unwrap();
    assert_eq!(registry.config.auth_token, Some("test-token-123".into()));
    registry.logout();
    assert!(registry.config.auth_token.is_none());
  }
}
