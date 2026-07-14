use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::{Arc, LazyLock, Mutex},
};

use deno_core::{
  ModuleLoadOptions, ModuleLoadReferrer, ModuleLoadResponse, ModuleLoader,
  ModuleResolveResponse, ModuleSource, ModuleSourceCode, ModuleSpecifier, ModuleType, ResolutionKind,
};
use deno_error::JsErrorBox;

use crate::permissions::Permissions;
use crate::transpiler::Transpiler;

// ── Import Map ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
struct ImportMap {
  imports: HashMap<String, String>,
  scopes: HashMap<String, HashMap<String, String>>,
}

impl ImportMap {
  fn auto_detect(start_dir: &Path) -> Self {
    let names = &["import_map.json", "import_map.jsonc"];
    let mut dir = Some(start_dir);
    while let Some(d) = dir {
      for name in names {
        let path = d.join(name);
        if path.exists() {
          if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
              return Self::from_json(&json);
            }
          }
        }
      }
      dir = d.parent();
    }
    // Check klyron.json for importMap field
    let klyron_json = start_dir.join("klyron.json");
    if klyron_json.exists() {
      if let Ok(content) = std::fs::read_to_string(&klyron_json) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
          if let Some(import_map) = json.get("importMap").and_then(|v| v.as_str()) {
            let imp_path = start_dir.join(import_map);
            if imp_path.exists() {
              if let Ok(imp_content) = std::fs::read_to_string(&imp_path) {
                if let Ok(imp_json) = serde_json::from_str::<serde_json::Value>(&imp_content) {
                  return Self::from_json(&imp_json);
                }
              }
            }
          }
          if let Some(imports) = json.get("imports").and_then(|v| v.as_object()) {
            let mut map = ImportMap::default();
            for (k, v) in imports {
              if let Some(s) = v.as_str() {
                map.imports.insert(k.clone(), s.to_string());
              }
            }
            return map;
          }
        }
      }
    }
    Self::default()
  }

  fn from_json(value: &serde_json::Value) -> Self {
    let mut map = ImportMap::default();
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
    map
  }

  fn resolve(&self, specifier: &str, referrer: Option<&str>) -> Option<String> {
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
    if let Some(value) = imports.get(specifier) {
      return Some(value.clone());
    }
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

static NODE_BUILTINS: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
  vec![
    ("assert", "ext:klyron_node/assert.js"),
    ("buffer", "ext:klyron_node/buffer.js"),
    ("child_process", "ext:klyron_node/child_process.js"),
    ("crypto", "ext:klyron_node/crypto.js"),
    ("events", "ext:klyron_node/events.js"),
    ("fs", "ext:klyron_node/fs.js"),
    ("os", "ext:klyron_node/os.js"),
    ("path", "ext:klyron_node/path.js"),
    ("process", "ext:klyron_node/process.js"),
    ("querystring", "ext:klyron_node/querystring.js"),
    ("stream", "ext:klyron_node/stream.js"),
    ("string_decoder", "ext:klyron_node/string_decoder.js"),
    ("url", "ext:klyron_node/url.js"),
    ("util", "ext:klyron_node/util.js"),
  ]
});

struct CachedModule {
  code: String,
  module_type: ModuleType,
}

pub struct KlyronModuleLoader {
  permissions: Arc<Permissions>,
  transpiler: Option<Transpiler>,
  cache: Arc<Mutex<Vec<(String, CachedModule)>>>,
  _npm_registry: String,
}

impl KlyronModuleLoader {
  pub fn new(permissions: Arc<Permissions>, transpiler: Option<Transpiler>) -> Self {
    let npm_registry = std::env::var("KLYRON_NPM_REGISTRY")
      .unwrap_or_else(|_| "https://registry.npmjs.org".to_string());
    Self { permissions, transpiler, cache: Arc::new(Mutex::new(Vec::new())), _npm_registry: npm_registry }
  }

  fn cache_get(&self, url: &str) -> Option<CachedModule> {
    self.cache.lock().ok().and_then(|c| {
      c.iter().find(|(k, _)| k == url).map(|(_, s)| CachedModule {
        code: s.code.clone(),
        module_type: s.module_type.clone(),
      })
    })
  }

  fn cache_set(&self, url: String, module: CachedModule) {
    if let Ok(mut c) = self.cache.lock() {
      c.push((url, module));
    }
  }

  fn resolve_file_path(&self, path: &Path) -> ModuleResolveResponse {
    let extensions = &[".js", ".ts", ".tsx", ".jsx", ".json", ".mjs", ".cjs"][..];
    if path.exists() && path.is_file() {
      return self.to_specifier(path);
    }
    for ext in extensions {
      let with_ext = path.with_extension(ext.trim_start_matches('.'));
      if with_ext.exists() && with_ext.is_file() {
        return self.to_specifier(&with_ext);
      }
    }
    for ext in extensions {
      let index = path.join(format!("index{ext}"));
      if index.exists() && index.is_file() {
        return self.to_specifier(&index);
      }
    }
    Err(JsErrorBox::generic(format!("Module not found: {}", path.display())))
  }

  fn to_specifier(&self, path: &Path) -> ModuleResolveResponse {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let url = format!("file://{}", canonical.display());
    Ok(ModuleSpecifier::parse(&url).unwrap())
  }

  fn resolve_node_modules(&self, name: &str, referrer_dir: &Path) -> ModuleResolveResponse {
    let mut dir = Some(referrer_dir);
    while let Some(current) = dir {
      let nm = current.join("node_modules").join(name);
      if nm.exists() {
        return self.resolve_file_path(&nm);
      }
      dir = current.parent();
    }
    Err(JsErrorBox::generic(format!("Cannot find module '{name}'")))
  }

  fn resolve_npm(&self, specifier: &str) -> ModuleResolveResponse {
    let package = specifier.strip_prefix("npm:").unwrap_or(specifier);
    // npm:package or npm:package@version or npm:@scope/package
    let (package_name, _version) = if let Some(idx) = package.find('@') {
      if package.starts_with('@') {
        // @scope/package@version
        let at_idx = package[1..].rfind('@').map(|i| i + 1);
        match at_idx {
          Some(pos) => (&package[..pos], Some(&package[pos + 1..])),
          None => (package, None),
        }
      } else {
        let pos = idx;
        (&package[..pos], Some(&package[pos + 1..]))
      }
    } else {
      (package, None)
    };
    // For now, resolve npm: specifiers by looking in node_modules
    self.resolve_node_modules(package_name, &std::env::current_dir().unwrap())
  }
}

impl ModuleLoader for KlyronModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> ModuleResolveResponse {
    // node: builtins -> ext:klyron_node/*.js
    if let Some(builtin) = specifier.strip_prefix("node:") {
      for (name, url) in NODE_BUILTINS.iter() {
        if *name == builtin {
          return Ok(ModuleSpecifier::parse(url).unwrap());
        }
      }
      return Err(JsErrorBox::generic(format!("Unknown node:builtin: {builtin}")));
    }

    // npm: specifiers
    if specifier.starts_with("npm:") {
      return self.resolve_npm(specifier);
    }

    // ext: / klyron: -> deno_core extension modules
    if specifier.starts_with("ext:") || specifier.starts_with("klyron:") {
      return Ok(ModuleSpecifier::parse(specifier).unwrap());
    }

    // file:// protocol
    if specifier.starts_with("file://") {
      return Ok(ModuleSpecifier::parse(specifier).unwrap());
    }

    // data: and blob: URLs
    if specifier.starts_with("data:") || specifier.starts_with("blob:") {
      return Ok(ModuleSpecifier::parse(specifier).unwrap());
    }

    // Relative imports
    if specifier.starts_with("./") || specifier.starts_with("../") {
      if referrer.starts_with("ext:") || referrer.starts_with("klyron:") {
        let base_url = ModuleSpecifier::parse(referrer).unwrap();
        return Ok(base_url.join(specifier).unwrap());
      }
      let base_path = if referrer.starts_with("file://") {
        ModuleSpecifier::parse(referrer)
          .ok()
          .and_then(|s| s.to_file_path().ok())
          .unwrap_or_else(|| std::env::current_dir().unwrap())
      } else {
        std::env::current_dir().unwrap()
      };
      let base_dir = if base_path.is_file() {
        base_path.parent().unwrap().to_path_buf()
      } else {
        base_path
      };
      return self.resolve_file_path(&base_dir.join(specifier));
    }

    if specifier.starts_with('/') {
      return self.resolve_file_path(&PathBuf::from(specifier));
    }

    // Bare specifier -> node_modules, then npm:
    self.resolve_node_modules(specifier, &std::env::current_dir().unwrap())
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    _maybe_referrer: Option<&ModuleLoadReferrer>,
    _options: ModuleLoadOptions,
  ) -> ModuleLoadResponse {
    let url = module_specifier.to_string();

    // ext: modules are handled by deno_core internally
    if url.starts_with("ext:") || url.starts_with("klyron:") {
      return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
        "ext: modules handled by deno_core: {url}"
      ))));
    }

    // Check cache first
    if let Some(cached) = self.cache_get(&url) {
      let code = ModuleSourceCode::String(cached.code.into());
      let src = ModuleSource::new(cached.module_type, code, module_specifier, None);
      return ModuleLoadResponse::Sync(Ok(src));
    }

    // data: URLs
    if url.starts_with("data:") {
      if let Some(content) = url.split(',').nth(1) {
        let decoded = urlencoding::decode(content).unwrap_or_else(|_| content.into());
        let code = ModuleSourceCode::String(decoded.to_string().into());
        let src = ModuleSource::new(
          ModuleType::JavaScript,
          code,
          module_specifier,
          None,
        );
        return ModuleLoadResponse::Sync(Ok(src));
      }
    }

    // File system
    if let Ok(path) = module_specifier.to_file_path() {
      if path.exists() {
        let path_str = path.to_string_lossy();
        if let Err(e) = self.permissions.check_read(&path_str) {
          return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(e)));
        }

        match std::fs::read_to_string(&path) {
          Ok(content) => {
            let is_ts = path
              .extension()
              .map(|e| e == "ts" || e == "tsx" || e == "mts")
              .unwrap_or(false);
            let is_json = path
              .extension()
              .map(|e| e == "json")
              .unwrap_or(false);
            let module_type = if is_json {
              ModuleType::Json
            } else {
              ModuleType::JavaScript
            };
            let code = if is_ts {
              if let Some(ref t) = self.transpiler {
                t.transpile(&content).unwrap_or(content)
              } else {
                content
              }
            } else {
              content
            };
            let cached = CachedModule {
              code: code.clone(),
              module_type: module_type.clone(),
            };
            self.cache_set(url, cached);
            let src = ModuleSource::new(
              module_type,
              ModuleSourceCode::String(code.into()),
              module_specifier,
              None,
            );
            return ModuleLoadResponse::Sync(Ok(src));
          }
          Err(e) => {
            return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
              "Read error {path:?}: {e}"
            ))));
          }
        }
      }
    }

    ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
      "Module not found: {url}"
    ))))
  }
}
