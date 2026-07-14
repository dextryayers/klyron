use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;
use url::Url;
use once_cell::sync::Lazy;

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum LoaderError {
  #[error("Module not found: {0}")]
  ModuleNotFound(String),
  #[error("Resolution error: {0}")]
  ResolutionError(String),
  #[error("Invalid specifier: {0}")]
  InvalidSpecifier(String),
  #[error("Unsupported scheme: {0}")]
  UnsupportedScheme(String),
  #[error("Import map error: {0}")]
  ImportMapError(String),
  #[error("Parse error: {0}")]
  ParseError(String),
}

// ── Module Kind ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleKind {
  Unknown,
  JavaScript,
  TypeScript,
  Mjs,
  Cjs,
  Json,
  Wasm,
  Node,
  Jsx,
  Tsx,
}

impl Default for ModuleKind {
  fn default() -> Self {
    Self::Unknown
  }
}

impl ModuleKind {
  pub fn from_extension(ext: &str) -> Self {
    match ext {
      ".js" | ".javascript" => Self::JavaScript,
      ".mjs" => Self::Mjs,
      ".cjs" => Self::Cjs,
      ".ts" => Self::TypeScript,
      ".tsx" => Self::Tsx,
      ".jsx" => Self::Jsx,
      ".json" => Self::Json,
      ".wasm" => Self::Wasm,
      ".node" => Self::Node,
      _ => Self::Unknown,
    }
  }

  pub fn extension(&self) -> &str {
    match self {
      Self::JavaScript => ".js",
      Self::Mjs => ".mjs",
      Self::Cjs => ".cjs",
      Self::TypeScript => ".ts",
      Self::Tsx => ".tsx",
      Self::Jsx => ".jsx",
      Self::Json => ".json",
      Self::Wasm => ".wasm",
      Self::Node => ".node",
      Self::Unknown => "",
    }
  }

  pub fn is_esm(&self) -> bool {
    matches!(self, Self::Mjs | Self::JavaScript | Self::TypeScript | Self::Jsx | Self::Tsx | Self::Wasm)
  }

  pub fn is_cjs(&self) -> bool {
    *self == Self::Cjs
  }
}

// ── Module Format ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleFormat {
  CommonJS,
  ESM,
  Unknown,
}

// ── Resolution Result ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ModuleResolution {
  pub resolved_url: Url,
  pub resolved_path: PathBuf,
  pub format: ModuleFormat,
  pub kind: ModuleKind,
  pub package_json: Option<Value>,
}

// ── Resolution Cache ──────────────────────────────────────────────────────────

static RESOLVE_CACHE: Lazy<Mutex<HashMap<String, Option<PathBuf>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn clear_resolve_cache() {
    if let Ok(mut cache) = RESOLVE_CACHE.lock() {
        cache.clear();
    }
}

pub fn get_resolve_cache_size() -> usize {
    RESOLVE_CACHE.lock().map(|c| c.len()).unwrap_or(0)
}

// ── Import Map ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct ImportMap {
  pub imports: HashMap<String, String>,
  pub scopes: HashMap<String, HashMap<String, String>>,
}

impl ImportMap {
  pub fn new() -> Self {
    Self {
      imports: HashMap::new(),
      scopes: HashMap::new(),
    }
  }

  pub fn from_json(value: &Value) -> Result<Self, LoaderError> {
    let mut map = Self::new();
    if let Some(imports) = value.get("imports").and_then(|v| v.as_object()) {
      for (k, v) in imports {
        if let Some(s) = v.as_str() {
          map.imports.insert(k.clone(), s.to_string());
        }
      }
    }
    if let Some(scopes) = value.get("scopes").and_then(|v| v.as_object()) {
      for (scope_key, scope_val) in scopes {
        if let Some(scope_obj) = scope_val.as_object() {
          let mut scope_map = HashMap::new();
          for (k, v) in scope_obj {
            if let Some(s) = v.as_str() {
              scope_map.insert(k.clone(), s.to_string());
            }
          }
          map.scopes.insert(scope_key.clone(), scope_map);
        }
      }
    }
    Ok(map)
  }

  pub fn resolve(&self, specifier: &str, referrer: Option<&str>) -> Option<String> {
    // Check scopes first
    if let Some(referrer_str) = referrer {
      for (scope_prefix, scope_imports) in &self.scopes {
        if referrer_str.starts_with(scope_prefix) {
          if let Some(result) = Self::match_imports(scope_imports, specifier) {
            return Some(result);
          }
        }
      }
    }
    Self::match_imports(&self.imports, specifier)
  }

  fn match_imports(imports: &HashMap<String, String>, specifier: &str) -> Option<String> {
    // Exact match
    if let Some(value) = imports.get(specifier) {
      return Some(value.clone());
    }
    // Prefix match with trailing /
    for (key, value) in imports {
      if let Some(prefix) = key.strip_suffix('/') {
        if specifier.starts_with(prefix) {
          let suffix = specifier.strip_prefix(prefix)?;
          let mapped = value.trim_end_matches('/').to_string() + suffix;
          return Some(mapped);
        }
      }
      if let Some(prefix) = key.strip_suffix('*') {
        if let Some(suffix) = specifier.strip_prefix(prefix) {
          let mapped = value.replace('*', suffix);
          return Some(mapped);
        }
      }
    }
    None
  }
}

// ── Module Resolver ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ModuleResolver {
  pub import_map: ImportMap,
  pub registry_urls: HashMap<String, String>,
}

impl Default for ModuleResolver {
  fn default() -> Self {
    Self::new()
  }
}

impl ModuleResolver {
  pub fn new() -> Self {
    let mut registry_urls = HashMap::new();
    registry_urls.insert("npm".into(), "https://registry.npmjs.org".into());
    registry_urls.insert("jsr".into(), "https://jsr.io".into());
    registry_urls.insert("deno".into(), "https://deno.land".into());

    Self {
      import_map: ImportMap::new(),
      registry_urls,
    }
  }

  // ── Format Detection ────────────────────────────────────────────────────────

  pub fn detect_format(path: &Path) -> ModuleFormat {
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    if filename.ends_with(".mjs") || filename.ends_with(".mts") {
      return ModuleFormat::ESM;
    }
    if filename.ends_with(".cjs") || filename.ends_with(".cts") {
      return ModuleFormat::CommonJS;
    }
    if filename.ends_with(".js") || filename.ends_with(".ts") || filename.ends_with(".jsx") || filename.ends_with(".tsx") {
      if let Some(dir) = path.parent() {
        let pkg_json = dir.join("package.json");
        if pkg_json.exists() {
          if let Ok(content) = std::fs::read_to_string(&pkg_json) {
            if let Ok(pkg) = serde_json::from_str::<Value>(&content) {
              if let Some(module_type) = pkg.get("type").and_then(|t| t.as_str()) {
                return if module_type == "module" {
                  ModuleFormat::ESM
                } else {
                  ModuleFormat::CommonJS
                };
              }
            }
          }
        }
      }
    }
    if filename.ends_with(".wasm") || filename.ends_with(".json") {
      return ModuleFormat::ESM;
    }
    ModuleFormat::CommonJS
  }

  pub fn detect_kind(path: &Path) -> ModuleKind {
    let ext = path
      .extension()
      .and_then(|e| e.to_str())
      .map(|e| format!(".{e}"))
      .unwrap_or_default();
    ModuleKind::from_extension(&ext)
  }

  pub fn auto_detect(path: &Path) -> (ModuleFormat, ModuleKind) {
    (Self::detect_format(path), Self::detect_kind(path))
  }

  // ── URL-Based Resolution ──────────────────────────────────────────────────

  pub fn resolve_specifier(specifier: &str, base: &Url) -> Result<Url, LoaderError> {
    if let Ok(url) = Url::parse(specifier) {
      let scheme = url.scheme();
      if matches!(scheme, "https" | "http" | "file" | "node" | "npm" | "data") {
        return Ok(url);
      }
      return Err(LoaderError::UnsupportedScheme(scheme.to_string()));
    }

    if specifier.starts_with("node:") {
      let node_spec = specifier.strip_prefix("node:").unwrap_or(specifier);
      return Url::parse(&format!("node:///{}", node_spec))
        .map_err(|e| LoaderError::InvalidSpecifier(e.to_string()));
    }

    if specifier.starts_with("file:") {
      let path = specifier.strip_prefix("file://").unwrap_or(specifier);
      return PathBuf::from(path).canonicalize().map_err(|e| {
        LoaderError::ModuleNotFound(format!("{path}: {e}"))
      }).and_then(|p| {
        Url::from_file_path(&p).map_err(|_| LoaderError::InvalidSpecifier(p.to_string_lossy().into()))
      });
    }

    base.join(specifier)
      .map_err(|e| LoaderError::InvalidSpecifier(format!("{specifier}: {e}")))
  }

  // ── Import Map Resolution ────────────────────────────────────────────────

  pub fn resolve_import_map(&self, specifier: &str, referrer: Option<&str>) -> Option<String> {
    self.import_map.resolve(specifier, referrer)
  }

  // ── Bare Specifier Resolution ────────────────────────────────────────────

  pub fn resolve_bare_specifier(specifier: &str, start: &Path) -> Result<PathBuf, LoaderError> {
    // Handle subpath imports: lodash/merge -> resolve lodash first, then find merge
    let (package_name, subpath) = Self::parse_package_specifier(specifier);

    let mut dir = Some(start);
    while let Some(d) = dir {
      let nm = d.join("node_modules").join(&package_name);
      if let Some(resolved) = Self::resolve_node_module_entry(&nm) {
        if let Some(sub) = &subpath {
          if let Some(parent) = resolved.parent() {
            let sub_path = parent.join(sub);
            // Try exact, then with extensions, then index files
            if sub_path.is_file() {
              return Ok(sub_path);
            }
            for ext in &[".js", ".mjs", ".cjs", ".json", ".ts", ".tsx", ".jsx"] {
              let with_ext = sub_path.with_extension(ext.trim_start_matches('.'));
              if with_ext.is_file() {
                return Ok(with_ext);
              }
            }
            for name in &["index.js", "index.mjs", "index.cjs", "index.ts", "index.tsx", "index.jsx"] {
              let idx = sub_path.join(name);
              if idx.is_file() {
                return Ok(idx);
              }
            }
            // Try node_modules of the resolved parent
            if let Some(pkg_node_modules) = parent.join("node_modules").parent() {
              let sub_nm = pkg_node_modules.join(sub);
              if let Some(sub_resolved) = Self::resolve_node_module_entry(&sub_nm) {
                return Ok(sub_resolved);
              }
            }
          }
          return Err(LoaderError::ModuleNotFound(format!(
            "Cannot find subpath '{sub}' in package '{package_name}'"
          )));
        }
        return Ok(resolved);
      }
      dir = d.parent();
    }
    // Try global node_modules
    if let Ok(home) = std::env::var("HOME") {
      let global = PathBuf::from(home).join(".node_modules").join(&package_name);
      if let Some(resolved) = Self::resolve_node_module_entry(&global) {
        return Ok(resolved);
      }
    }
    Err(LoaderError::ModuleNotFound(format!(
      "Cannot find bare specifier '{specifier}'"
    )))
  }

  fn parse_package_specifier(specifier: &str) -> (String, Option<String>) {
    if let Some(slash_idx) = specifier.find('/') {
      if specifier.starts_with('@') {
        // @scope/package or @scope/package/subpath
        if let Some(next_slash) = specifier[slash_idx + 1..].find('/') {
          let pkg_end = slash_idx + 1 + next_slash;
          let pkg = &specifier[..pkg_end];
          let sub = &specifier[pkg_end + 1..];
          (pkg.to_string(), Some(sub.to_string()))
        } else {
          (specifier.to_string(), None)
        }
      } else {
        let pkg = &specifier[..slash_idx];
        let sub = &specifier[slash_idx + 1..];
        (pkg.to_string(), Some(sub.to_string()))
      }
    } else {
      (specifier.to_string(), None)
    }
  }

  fn resolve_node_module_entry(dir: &Path) -> Option<PathBuf> {
    if dir.is_file() {
      return Some(dir.to_path_buf());
    }
    if dir.is_dir() {
      // Check package.json for exports/main
      let pkg_json = dir.join("package.json");
      if pkg_json.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pkg_json) {
          if let Ok(pkg) = serde_json::from_str::<Value>(&content) {
            if let Some(entry) = Self::resolve_package_json_entry(dir, &pkg) {
              return Some(entry);
            }
          }
        }
      }
      // index.js fallback
      for name in &["index.js", "index.mjs", "index.cjs", "index.json", "index.ts", "index.jsx", "index.tsx"] {
        let idx = dir.join(name);
        if idx.is_file() {
          return Some(idx);
        }
      }
    }
    // Try with extensions
    for ext in &[".js", ".mjs", ".cjs", ".json", ".ts", ".jsx", ".tsx", ".wasm", ".node"] {
      let with_ext = dir.with_extension(ext.trim_start_matches('.'));
      if with_ext.is_file() {
        return Some(with_ext);
      }
    }
    None
  }

  // ── Package.json Resolution ─────────────────────────────────────────────

  pub fn resolve_package_json(pkg_dir: &Path) -> Result<Value, LoaderError> {
    let pkg_json = pkg_dir.join("package.json");
    let content = std::fs::read_to_string(&pkg_json)
      .map_err(|e| LoaderError::ModuleNotFound(format!("package.json not found: {e}")))?;
    serde_json::from_str(&content)
      .map_err(|e| LoaderError::ParseError(format!("Invalid package.json: {e}")))
  }

  pub fn resolve_package_json_entry(pkg_dir: &Path, pkg: &Value) -> Option<PathBuf> {
    // Exports field (Node.js conditional exports)
    if let Some(exports) = pkg.get("exports") {
      if let Some(s) = exports.as_str() {
        let entry = pkg_dir.join(s);
        if entry.is_file() {
          return Some(entry);
        }
      }
      if let Some(obj) = exports.as_object() {
        // Check "." shorthand
        for key in &[".", "."] {
          if let Some(val) = obj.get(*key) {
            return Self::resolve_export_value(pkg_dir, val);
          }
        }
        // Check named exports like "./*"
        for val in obj.values() {
          if let Some(entry) = Self::resolve_export_value(pkg_dir, val) {
            return Some(entry);
          }
        }
      }
    }

    // "imports" field (package import maps)
    if let Some(imports) = pkg.get("imports").and_then(|v| v.as_object()) {
      for (_key, val) in imports {
        if let Some(s) = val.as_str() {
          let entry = pkg_dir.join(s);
          if entry.is_file() {
            return Some(entry);
          }
        }
      }
    }

    // Main field fallback
    if let Some(main) = pkg.get("main").and_then(|m| m.as_str()) {
      let entry = pkg_dir.join(main);
      if entry.is_file() {
        return Some(entry);
      }
      for ext in &[".js", ".mjs", ".cjs", ".json", ".ts"] {
        let with_ext = entry.with_extension(ext.trim_start_matches('.'));
        if with_ext.is_file() {
          return Some(with_ext);
        }
      }
      let idx = entry.join("index.js");
      if idx.is_file() {
        return Some(idx);
      }
    }

    None
  }

  fn resolve_export_value(pkg_dir: &Path, val: &Value) -> Option<PathBuf> {
    match val {
      Value::String(s) => {
        let entry = pkg_dir.join(s);
        if entry.is_file() {
          return Some(entry);
        }
        None
      }
      Value::Object(obj) => {
        // Conditional exports - prefer import, then require, then default
        for key in &["import", "require", "default"] {
          if let Some(val) = obj.get(*key) {
            if let Some(entry) = Self::resolve_export_value(pkg_dir, val) {
              return Some(entry);
            }
          }
        }
        None
      }
      _ => None,
    }
  }

  // ── Full Resolution ──────────────────────────────────────────────────────

  pub fn resolve(&self, specifier: &str, base: &Path) -> Result<ModuleResolution, LoaderError> {
    // Check cache first
    let cache_key = format!("{}:{}", specifier, base.display());
    {
      if let Ok(cache) = RESOLVE_CACHE.lock() {
        if let Some(Some(cached_path)) = cache.get(&cache_key) {
          let format = Self::detect_format(cached_path);
          let kind = Self::detect_kind(cached_path);
          let url = Url::from_file_path(cached_path).unwrap_or_else(|_| Url::parse("file:///").unwrap());
          return Ok(ModuleResolution {
            resolved_url: url,
            resolved_path: cached_path.clone(),
            format,
            kind,
            package_json: None,
          });
        }
      }
    }

    let result = self.resolve_inner(specifier, base);

    // Cache success results
    if let Ok(ref resolution) = result {
      if let Ok(mut cache) = RESOLVE_CACHE.lock() {
        cache.insert(cache_key, Some(resolution.resolved_path.clone()));
      }
    }

    result
  }

  fn resolve_inner(&self, specifier: &str, base: &Path) -> Result<ModuleResolution, LoaderError> {
    // 1. Try import map first
    let base_url = Url::from_file_path(base).ok();
    let referrer = base_url.as_ref().map(|u| u.as_str());

    if let Some(mapped) = self.resolve_import_map(specifier, referrer) {
      let url = Url::parse(&mapped)
        .map_err(|e| LoaderError::InvalidSpecifier(format!("Import map mapped to invalid URL: {e}")))?;
      let path = if url.scheme() == "file" {
        url.to_file_path()
          .unwrap_or_else(|_| PathBuf::from(url.path()))
      } else {
        PathBuf::from(url.path())
      };
      let format = Self::detect_format(&path);
      let kind = Self::detect_kind(&path);
      return Ok(ModuleResolution {
        resolved_url: url,
        resolved_path: path,
        format,
        kind,
        package_json: None,
      });
    }

    // 2. URL specifier
    if specifier.starts_with("https://") || specifier.starts_with("http://") {
      let url =
        Url::parse(specifier).map_err(|e| LoaderError::InvalidSpecifier(e.to_string()))?;
      let path = PathBuf::from(url.path());
      let format = Self::detect_format(&path);
      let kind = Self::detect_kind(&path);
      return Ok(ModuleResolution {
        resolved_url: url,
        resolved_path: path,
        format,
        kind,
        package_json: None,
      });
    }

    if specifier.starts_with("file://") {
      let url = Url::parse(specifier).map_err(|e| LoaderError::InvalidSpecifier(e.to_string()))?;
      let path = url
        .to_file_path()
        .map_err(|_| LoaderError::InvalidSpecifier(format!("Invalid file URL: {specifier}")))?;
      let format = Self::detect_format(&path);
      let kind = Self::detect_kind(&path);
      return Ok(ModuleResolution {
        resolved_url: url,
        resolved_path: path,
        format,
        kind,
        package_json: None,
      });
    }

    if specifier.starts_with("node:") {
      let name = specifier.strip_prefix("node:").unwrap_or(specifier);
      let url = Url::parse(&format!("node:///{name}"))
        .map_err(|e| LoaderError::InvalidSpecifier(e.to_string()))?;
      return Ok(ModuleResolution {
        resolved_url: url,
        resolved_path: PathBuf::from(specifier),
        format: ModuleFormat::ESM,
        kind: ModuleKind::Node,
        package_json: None,
      });
    }

    if specifier.starts_with("npm:") {
      let package = specifier.strip_prefix("npm:").unwrap_or(specifier);
      let start = if base.is_file() { base.parent().unwrap_or(base) } else { base };
      let resolved = Self::resolve_bare_specifier(package, start)?;
      let url = Url::from_file_path(&resolved).unwrap_or_else(|_| Url::parse("file:///").unwrap());
      let format = Self::detect_format(&resolved);
      let kind = Self::detect_kind(&resolved);
      return Ok(ModuleResolution {
        resolved_url: url,
        resolved_path: resolved,
        format,
        kind,
        package_json: None,
      });
    }

    // 3. data: URLs
    if specifier.starts_with("data:") {
      let url = Url::parse(specifier).map_err(|e| LoaderError::InvalidSpecifier(e.to_string()))?;
      return Ok(ModuleResolution {
        resolved_url: url.clone(),
        resolved_path: PathBuf::from(url.path()),
        format: ModuleFormat::ESM,
        kind: ModuleKind::JavaScript,
        package_json: None,
      });
    }

    // 4. Relative or absolute path
    if specifier.starts_with('.') || specifier.starts_with('/') {
      let base_dir = if base.is_file() {
        base.parent().unwrap_or(base)
      } else {
        base
      };
      let candidate = base_dir.join(specifier);
      if candidate.is_file() {
        let url = Url::from_file_path(&candidate)
          .unwrap_or_else(|_| Url::parse("file:///").unwrap());
        let format = Self::detect_format(&candidate);
        let kind = Self::detect_kind(&candidate);
        return Ok(ModuleResolution {
          resolved_url: url,
          resolved_path: candidate,
          format,
          kind,
          package_json: None,
        });
      }
      // Try with extensions
      for ext in &[
        ".js", ".mjs", ".cjs", ".json", ".ts", ".tsx", ".jsx", ".wasm", ".node",
      ] {
        let with_ext = candidate.with_extension(ext.trim_start_matches('.'));
        if with_ext.is_file() {
          let url = Url::from_file_path(&with_ext)
            .unwrap_or_else(|_| Url::parse("file:///").unwrap());
          let format = Self::detect_format(&with_ext);
          let kind = Self::detect_kind(&with_ext);
          return Ok(ModuleResolution {
            resolved_url: url,
            resolved_path: with_ext,
            format,
            kind,
            package_json: None,
          });
        }
      }
      // Index files
      for name in &[
        "index.js", "index.mjs", "index.cjs", "index.json", "index.ts", "index.tsx", "index.jsx",
      ] {
        let idx = candidate.join(name);
        if idx.is_file() {
          let url =
            Url::from_file_path(&idx).unwrap_or_else(|_| Url::parse("file:///").unwrap());
          let format = Self::detect_format(&idx);
          let kind = Self::detect_kind(&idx);
          return Ok(ModuleResolution {
            resolved_url: url,
            resolved_path: idx,
            format,
            kind,
            package_json: None,
          });
        }
      }
      // Package.json in directory
      if candidate.is_dir() {
        let pkg_json = candidate.join("package.json");
        if pkg_json.is_file() {
          if let Ok(pkg) = Self::resolve_package_json(&candidate) {
            if let Some(entry) = Self::resolve_package_json_entry(&candidate, &pkg) {
              let url = Url::from_file_path(&entry)
                .unwrap_or_else(|_| Url::parse("file:///").unwrap());
              let format = Self::detect_format(&entry);
              let kind = Self::detect_kind(&entry);
              return Ok(ModuleResolution {
                resolved_url: url,
                resolved_path: entry,
                format,
                kind,
                package_json: Some(pkg),
              });
            }
          }
        }
      }
      return Err(LoaderError::ModuleNotFound(format!("Cannot resolve '{specifier}' from '{}'", base.display())));
    }

    // 5. Bare specifier (node_modules)
    let start = if base.is_file() {
      base.parent().unwrap_or(base)
    } else {
      base
    };
    let resolved = Self::resolve_bare_specifier(specifier, start)?;
    let url =
      Url::from_file_path(&resolved).unwrap_or_else(|_| Url::parse("file:///").unwrap());
    let format = Self::detect_format(&resolved);
    let kind = Self::detect_kind(&resolved);

    let pkg_json = 'find_pkg: {
      if resolved.starts_with("node_modules") || resolved.to_string_lossy().contains("node_modules") {
        let mut parent = resolved.parent();
        while let Some(dir) = parent {
          let pj = dir.join("package.json");
          if pj.is_file() {
            if let Ok(pkg) = Self::resolve_package_json(dir) {
              break 'find_pkg Some(pkg);
            }
          }
          if dir.ends_with("node_modules") {
            break 'find_pkg None;
          }
          parent = dir.parent();
        }
      }
      None
    };

    Ok(ModuleResolution {
      resolved_url: url,
      resolved_path: resolved,
      format,
      kind,
      package_json: pkg_json,
    })
  }
}

// ── CJS ↔ ESM Transpile ──────────────────────────────────────────────────────

pub fn cjs_to_esm(source: &str) -> String {
  let mut out = String::with_capacity(source.len() + 128);
  let lines: Vec<&str> = source.lines().collect();
  let mut i = 0;
  while i < lines.len() {
    let line = lines[i];
    let trimmed = line.trim();

    if trimmed.starts_with("module.exports = ") || trimmed == "module.exports =" {
      let prefix = &line[..line.len() - trimmed.len()];
      let rest = trimmed
        .strip_prefix("module.exports = ")
        .unwrap_or("");
      if rest.is_empty() {
        out.push_str(&format!("{prefix}export default "));
      } else if rest.starts_with('{') || rest.starts_with('[') || rest.starts_with('`') {
        out.push_str(&format!("{prefix}export default {rest}"));
      } else {
        out.push_str(&format!("{prefix}export default {rest}"));
      }
      out.push('\n');
    } else if let Some(rest) = trimmed.strip_prefix("exports.") {
      if let Some((name, _val)) = rest.split_once(" = ") {
        let prefix = &line[..line.len() - trimmed.len()];
        out.push_str(&format!("{prefix}export const {name} = {_val}"));
        out.push('\n');
      } else {
        out.push_str(line);
        out.push('\n');
      }
    } else if let Some(rest) = trimmed.strip_prefix("require(") {
      if let Some(arg) = rest.strip_suffix(")") {
        let prefix = &line[..line.len() - trimmed.len()];
        if line.trim().ends_with(';') {
          out.push_str(&format!("{prefix}import \"{}\";", arg.trim_matches(&['"', '\'', '`', ';'] as &[_])));
          out.push('\n');
        } else {
          out.push_str(&format!("{prefix}import(\"{}\")", arg.trim_matches(&['"', '\'', '`'] as &[_])));
          out.push('\n');
        }
      } else {
        out.push_str(line);
        out.push('\n');
      }
    } else if trimmed.starts_with("__dirname") || trimmed.starts_with("__filename") {
      // Keep as-is - these are available in CJS
      out.push_str(line);
      out.push('\n');
    } else {
      out.push_str(line);
      out.push('\n');
    }
    i += 1;
  }
  out
}

pub fn esm_to_cjs(source: &str) -> String {
  let mut out = String::with_capacity(source.len() + 128);
  let lines: Vec<&str> = source.lines().collect();
  let mut i = 0;
  while i < lines.len() {
    let line = lines[i];
    let trimmed = line.trim();

    if let Some(rest) = trimmed.strip_prefix("export default ") {
      let prefix = &line[..line.len() - trimmed.len()];
      out.push_str(&format!("{prefix}module.exports = {rest}"));
      out.push('\n');
    } else if let Some(rest) = trimmed.strip_prefix("export const ") {
      if let Some((name_val, _val)) = rest.split_once(" = ") {
        let prefix = &line[..line.len() - trimmed.len()];
        out.push_str(&format!("{prefix}exports.{name_val} = {_val}"));
        out.push('\n');
      } else {
        out.push_str(line);
        out.push('\n');
      }
    } else if let Some(rest) = trimmed.strip_prefix("export function ") {
      let prefix = &line[..line.len() - trimmed.len()];
      // exports.name = function
      if let Some((name, _args)) = rest.split_once('(') {
        let name = name.trim();
        out.push_str(&format!("{prefix}function {name}"));
        // Get rest of function args/body until matching }
        let mut j = i;
        let mut depth = 0;
        let mut body = String::new();
        while j < lines.len() {
          body.push_str(lines[j]);
          body.push('\n');
          for c in lines[j].chars() {
            match c {
              '{' => depth += 1,
              '}' => depth -= 1,
              _ => {}
            }
          }
          if depth <= 0 {
            break;
          }
          j += 1;
        }
        out.push_str(&body);
        out.push_str(&format!("{prefix}exports.{name} = {name};\n"));
        i = j;
      } else {
        out.push_str(line);
        out.push('\n');
      }
    } else if let Some(rest) = trimmed.strip_prefix("import(") {
      if let Some((spec, _rest)) = rest.split_once(')') {
        let prefix = &line[..line.len() - trimmed.len()];
        out.push_str(&format!(
          "{prefix}require({}))",
          spec.trim_matches(&['"', '\'', '`'] as &[_])
        ));
        out.push('\n');
      } else {
        out.push_str(line);
        out.push('\n');
      }
    } else if trimmed.starts_with("import ") || trimmed.starts_with("import\t") {
      // Static imports
      if let Some(spec) = trimmed.strip_prefix("import ") {
        if let Some(module) = spec.strip_prefix('"').or_else(|| spec.strip_prefix('\'')) {
          if let Some(name) = module.split('"').next().or_else(|| module.split('\'').next()) {
            let prefix = &line[..line.len() - trimmed.len()];
            out.push_str(&format!("{prefix}const {} = require('{name}');\n", "mod"));
            continue;
          }
        }
      }
      out.push_str(line);
      out.push('\n');
    } else {
      out.push_str(line);
      out.push('\n');
    }
    i += 1;
  }
  out
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_module_kind_from_extension() {
    assert_eq!(ModuleKind::from_extension(".mjs"), ModuleKind::Mjs);
    assert_eq!(ModuleKind::from_extension(".cjs"), ModuleKind::Cjs);
    assert_eq!(ModuleKind::from_extension(".json"), ModuleKind::Json);
    assert_eq!(ModuleKind::from_extension(".wasm"), ModuleKind::Wasm);
    assert_eq!(ModuleKind::from_extension(".node"), ModuleKind::Node);
    assert_eq!(ModuleKind::from_extension(".jsx"), ModuleKind::Jsx);
    assert_eq!(ModuleKind::from_extension(".tsx"), ModuleKind::Tsx);
    assert_eq!(ModuleKind::from_extension(".ts"), ModuleKind::TypeScript);
    assert_eq!(ModuleKind::from_extension(".js"), ModuleKind::JavaScript);
  }

  #[test]
  fn test_module_kind_is_esm() {
    assert!(ModuleKind::Mjs.is_esm());
    assert!(ModuleKind::Wasm.is_esm());
    assert!(!ModuleKind::Cjs.is_esm());
    assert!(!ModuleKind::Json.is_esm());
  }

  #[test]
  fn test_import_map_exact() {
    let mut map = ImportMap::new();
    map.imports.insert("preact".into(), "https://esm.sh/preact".into());
    assert_eq!(
      map.resolve("preact", None),
      Some("https://esm.sh/preact".into())
    );
  }

  #[test]
  fn test_import_map_prefix() {
    let mut map = ImportMap::new();
    map.imports
      .insert("std/".into(), "https://deno.land/std@0.224.0/".into());
    assert_eq!(
      map.resolve("std/fs/mod.ts", None),
      Some("https://deno.land/std@0.224.0/fs/mod.ts".into())
    );
  }

  #[test]
  fn test_import_map_wildcard() {
    let mut map = ImportMap::new();
    map.imports.insert("std/*".into(), "https://deno.land/std@0.224.0/*".into());
    assert_eq!(
      map.resolve("std/fs", None),
      Some("https://deno.land/std@0.224.0/fs".into())
    );
  }

  #[test]
  fn test_import_map_scopes() {
    let mut map = ImportMap::new();
    map.imports.insert("lodash".into(), "https://esm.sh/lodash".into());
    let mut scope = HashMap::new();
    scope.insert("lodash".into(), "https://cdn.skypack.dev/lodash".into());
    map.scopes.insert("/app/".into(), scope);

    assert_eq!(
      map.resolve("lodash", Some("/app/index.ts")),
      Some("https://cdn.skypack.dev/lodash".into())
    );
    assert_eq!(
      map.resolve("lodash", Some("/other/index.ts")),
      Some("https://esm.sh/lodash".into())
    );
  }

  #[test]
  fn test_import_map_from_json() {
    let json: Value = serde_json::from_str(r#"{
      "imports": {
        "preact": "https://esm.sh/preact",
        "std/": "https://deno.land/std@0.224.0/"
      }
    }"#)
    .unwrap();
    let map = ImportMap::from_json(&json).unwrap();
    assert_eq!(map.resolve("preact", None), Some("https://esm.sh/preact".into()));
    assert_eq!(
      map.resolve("std/fs/mod.ts", None),
      Some("https://deno.land/std@0.224.0/fs/mod.ts".into())
    );
  }

  #[test]
  fn test_resolve_bare_specifier() {
    let dir = std::env::temp_dir().join("_klyron_loader_bare");
    let _ = std::fs::create_dir_all(&dir.join("node_modules").join("test-pkg"));
    std::fs::write(
      dir.join("node_modules/test-pkg/package.json"),
      r#"{"main":"index.js"}"#,
    )
    .unwrap();
    std::fs::write(dir.join("node_modules/test-pkg/index.js"), "").unwrap();
    let result = ModuleResolver::resolve_bare_specifier("test-pkg", &dir);
    assert!(result.is_ok());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_resolve_bare_specifier_not_found() {
    let result = ModuleResolver::resolve_bare_specifier("nonexistent-pkg-xyz", Path::new("/tmp"));
    assert!(result.is_err());
  }

  #[test]
  fn test_detect_format_mjs() {
    assert_eq!(
      ModuleResolver::detect_format(Path::new("/test/file.mjs")),
      ModuleFormat::ESM
    );
  }

  #[test]
  fn test_detect_format_cjs() {
    assert_eq!(
      ModuleResolver::detect_format(Path::new("/test/file.cjs")),
      ModuleFormat::CommonJS
    );
  }

  #[test]
  fn test_cjs_to_esm() {
    let cjs = r#"const path = require("path");
module.exports = { foo: 1 };
exports.bar = 2;"#;
    let esm = cjs_to_esm(cjs);
    assert!(esm.contains("export default"));
    assert!(!esm.contains("module.exports"));
  }

  #[test]
  fn test_esm_to_cjs() {
    let esm = r#"import { readFile } from "fs";
export default { foo: 1 };
export const bar = 2;"#;
    let cjs = esm_to_cjs(esm);
    assert!(cjs.contains("module.exports"));
    assert!(!cjs.contains("export default"));
  }

  #[test]
  fn test_resolve_specifier_https() {
    let url = Url::parse("file:///app/index.js").unwrap();
    let result = ModuleResolver::resolve_specifier("https://esm.sh/preact", &url);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().scheme(), "https");
  }

  #[test]
  fn test_resolve_specifier_node() {
    let url = Url::parse("file:///app/index.js").unwrap();
    let result = ModuleResolver::resolve_specifier("node:fs", &url);
    assert!(result.is_ok());
  }

  #[test]
  fn test_detect_kind() {
    assert_eq!(ModuleResolver::detect_kind(Path::new("f.mjs")), ModuleKind::Mjs);
    assert_eq!(ModuleResolver::detect_kind(Path::new("f.cjs")), ModuleKind::Cjs);
    assert_eq!(ModuleResolver::detect_kind(Path::new("f.tsx")), ModuleKind::Tsx);
    assert_eq!(ModuleResolver::detect_kind(Path::new("f.jsx")), ModuleKind::Jsx);
    assert_eq!(ModuleResolver::detect_kind(Path::new("f.node")), ModuleKind::Node);
    assert_eq!(ModuleResolver::detect_kind(Path::new("f.wasm")), ModuleKind::Wasm);
  }

  #[test]
  fn test_resolver_import_map_path() {
    let resolver = ModuleResolver::new();
    let dir = std::env::temp_dir().join("_klyron_loader_map");
    let _ = std::fs::create_dir_all(&dir);
    let entry = dir.join("index.js");
    std::fs::write(&entry, "").unwrap();
    let result = resolver.resolve("./index.js", &dir);
    assert!(result.is_ok());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_resolver_not_found() {
    let resolver = ModuleResolver::new();
    let result = resolver.resolve("./nonexistent_file_xyz.js", Path::new("/tmp"));
    assert!(result.is_err());
  }

  #[test]
  fn test_package_json_exports() {
    let dir = std::env::temp_dir().join("_klyron_pkg_exports");
    let _ = std::fs::create_dir_all(&dir);
    let pkg: Value = serde_json::from_str(r#"{
      "exports": {
        ".": "./dist/index.js",
        "./utils": "./dist/utils.js"
      }
    }"#)
    .unwrap();
    std::fs::write(dir.join("package.json"), serde_json::to_string(&pkg).unwrap()).unwrap();
    std::fs::create_dir_all(dir.join("dist")).unwrap();
    std::fs::write(dir.join("dist/index.js"), "").unwrap();
    let entry = ModuleResolver::resolve_package_json_entry(&dir, &pkg);
    assert!(entry.is_some());
    assert!(entry.unwrap().ends_with("dist/index.js"));
    let _ = std::fs::remove_dir_all(&dir);
  }
}
