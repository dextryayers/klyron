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
pub struct ImportMap {
  pub imports: HashMap<String, String>,
  pub scopes: HashMap<String, HashMap<String, String>>,
}

impl ImportMap {
  pub fn auto_detect(start_dir: &Path) -> Self {
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

  pub fn from_json(value: &serde_json::Value) -> Self {
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
    ("assert/strict", "ext:klyron_node/assert_strict.js"),
    ("buffer", "ext:klyron_node/buffer.js"),
    ("child_process", "ext:klyron_node/child_process.js"),
    ("cluster", "ext:klyron_node/cluster.js"),
    ("console", "ext:klyron_node/console.js"),
    ("crypto", "ext:klyron_node/crypto.js"),
    ("dgram", "ext:klyron_node/dgram.js"),
    ("diagnostics_channel", "ext:klyron_node/diagnostics_channel.js"),
    ("dns", "ext:klyron_node/dns.js"),
    ("domain", "ext:klyron_node/domain.js"),
    ("events", "ext:klyron_node/events.js"),
    ("fs", "ext:klyron_node/fs.js"),
    ("fs/promises", "ext:klyron_node/fs_promises.js"),
    ("http", "ext:klyron_node/http.js"),
    ("http2", "ext:klyron_node/http2.js"),
    ("https", "ext:klyron_node/https.js"),
    ("inspector", "ext:klyron_node/inspector.js"),
    ("module", "ext:klyron_node/module.js"),
    ("net", "ext:klyron_node/net.js"),
    ("os", "ext:klyron_node/os.js"),
    ("path", "ext:klyron_node/path.js"),
    ("path/posix", "ext:klyron_node/path_posix.js"),
    ("path/win32", "ext:klyron_node/path_win32.js"),
    ("perf_hooks", "ext:klyron_node/perf_hooks.js"),
    ("process", "ext:klyron_node/process.js"),
    ("punycode", "ext:klyron_node/punycode.js"),
    ("querystring", "ext:klyron_node/querystring.js"),
    ("readline", "ext:klyron_node/readline.js"),
    ("repl", "ext:klyron_node/repl.js"),
    ("stream", "ext:klyron_node/stream.js"),
    ("stream/promises", "ext:klyron_node/stream_promises.js"),
    ("stream/web", "ext:klyron_node/stream_web.js"),
    ("string_decoder", "ext:klyron_node/string_decoder.js"),
    ("timers", "ext:klyron_node/timers.js"),
    ("timers/promises", "ext:klyron_node/timers_promises.js"),
    ("tls", "ext:klyron_node/tls.js"),
    ("trace_events", "ext:klyron_node/trace_events.js"),
    ("tty", "ext:klyron_node/tty.js"),
    ("url", "ext:klyron_node/url.js"),
    ("util", "ext:klyron_node/util.js"),
    ("util/types", "ext:klyron_node/util_types.js"),
    ("v8", "ext:klyron_node/v8.js"),
    ("vm", "ext:klyron_node/vm.js"),
    ("worker_threads", "ext:klyron_node/worker_threads.js"),
    ("zlib", "ext:klyron_node/zlib.js"),
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
  import_map: ImportMap,
}

impl KlyronModuleLoader {
  pub fn new(permissions: Arc<Permissions>, transpiler: Option<Transpiler>) -> Self {
    let npm_registry = std::env::var("KLYRON_NPM_REGISTRY")
      .unwrap_or_else(|_| "https://registry.npmjs.org".to_string());
    let import_map = ImportMap::auto_detect(&std::env::current_dir().unwrap_or_default());
    Self { permissions, transpiler, cache: Arc::new(Mutex::new(Vec::new())), _npm_registry: npm_registry, import_map }
  }

  pub fn resolve_import_map(&self, specifier: &str, referrer: Option<&str>) -> Option<String> {
    self.import_map.resolve(specifier, referrer)
  }

  pub fn set_import_map(&mut self, import_map: ImportMap) {
    self.import_map = import_map;
  }

  pub fn get_import_map(&self) -> &ImportMap {
    &self.import_map
  }

  pub fn detect_import_map(start_dir: &Path) -> ImportMap {
    ImportMap::auto_detect(start_dir)
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
        return self.resolve_package_entry(&nm, name);
      }
      dir = current.parent();
    }
    Err(JsErrorBox::generic(format!("Cannot find module '{name}'")))
  }

  /// Determine the directory to start node_modules resolution from: the
  /// referrer's parent when the referrer is a file:// URL, otherwise the
  /// current working directory.
  fn referrer_search_dir(&self, referrer: &str) -> PathBuf {
    if referrer.starts_with("file://") {
      if let Ok(spec) = ModuleSpecifier::parse(referrer) {
        if let Ok(p) = spec.to_file_path() {
          if p.is_file() {
            return p.parent().unwrap_or(&p).to_path_buf();
          }
          return p;
        }
      }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
  }

  /// Resolve a package directory to its entry point using package.json
  /// `exports`, then `main`, then `module`, then index files. Honors the
  /// Node.js resolution algorithm (exports takes precedence over main).
  fn resolve_package_entry(&self, pkg_dir: &Path, _name: &str) -> ModuleResolveResponse {
    let pkg_json_path = pkg_dir.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
      if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
        // 1. "exports" (string or conditional map)
        if let Some(exports) = pkg.get("exports") {
          if let Some(target) = self.resolve_exports_entry(exports, pkg_dir) {
            return target;
          }
        }
        // 2. "main" / "module" (module preferred for ESM)
        for key in ["module", "main"] {
          if let Some(rel) = pkg.get(key).and_then(|v| v.as_str()) {
            let candidate = pkg_dir.join(rel);
            if candidate.exists() {
              return self.resolve_file_path(&candidate);
            }
          }
        }
      }
    }
    // 3. Fallback: index files inside the package dir
    self.resolve_file_path(pkg_dir)
  }

  /// Given the `exports` field value, pick the right target for the
  /// default import condition ("./"). Supports a plain string, a
  /// condition map ({".": ...} or {import/require/default}), and a
  /// nested map keyed by subpath.
  fn resolve_exports_entry(
    &self,
    exports: &serde_json::Value,
    pkg_dir: &Path,
  ) -> Option<ModuleResolveResponse> {
    // exports is a condition map keyed by subpath, e.g. {".": "./index.js"}
    if let Some(obj) = exports.as_object() {
      // Allow `exports` to be the package-level map directly.
      if let Some(root) = obj.get(".") {
        return self.pick_export_condition(root, pkg_dir);
      }
      // Otherwise pick the first subpath entry as the default entry.
      if let Some((_, first)) = obj.iter().next() {
        return self.pick_export_condition(first, pkg_dir);
      }
      return None;
    }
    // exports is a plain string
    if let Some(s) = exports.as_str() {
      let rel = s.trim_start_matches("./");
      return Some(self.resolve_file_path(&pkg_dir.join(rel)));
    }
    None
  }

  /// Pick the appropriate target from a conditional exports entry, preferring
  /// "import" (ESM), then "default", then "require".
  fn pick_export_condition(
    &self,
    entry: &serde_json::Value,
    pkg_dir: &Path,
  ) -> Option<ModuleResolveResponse> {
    if let Some(s) = entry.as_str() {
      let rel = s.trim_start_matches("./");
      return Some(self.resolve_file_path(&pkg_dir.join(rel)));
    }
    if let Some(obj) = entry.as_object() {
      for cond in ["import", "default", "require"] {
        if let Some(target) = obj.get(cond).and_then(|v| v.as_str()) {
          let rel = target.trim_start_matches("./");
          return Some(self.resolve_file_path(&pkg_dir.join(rel)));
        }
      }
    }
    None
  }

  fn resolve_npm(&self, specifier: &str, referrer: &str) -> ModuleResolveResponse {
    let package = specifier.strip_prefix("npm:").unwrap_or(specifier);
    // npm:package, npm:package@version, npm:@scope/package, npm:package/subpath
    let (package_name, subpath) = if let Some(slash_idx) = package.find('/') {
      // check if it's @scope/package or package/subpath
      if package.starts_with('@') {
        // @scope/package or @scope/package/subpath or @scope/package@version
        let after_scope = &package[slash_idx + 1..];
        let (rest, sub) = if let Some(next_slash) = after_scope.find('/') {
          let _pkg_base = &package[..slash_idx + 1 + next_slash];
          (&package[..slash_idx + 1 + next_slash], Some(&after_scope[next_slash + 1..]))
        } else {
          (package, None)
        };
        if let Some(ver_sep) = rest.rfind('@') {
          if ver_sep > slash_idx + 1 {
            let name = &rest[..ver_sep];
            let _ver = &rest[ver_sep + 1..];
            (name, sub)
          } else {
            (rest, sub)
          }
        } else {
          (rest, sub)
        }
      } else {
        // package/subpath or package@version/subpath
        if let Some(ver_sep) = package.find('@') {
          let name = &package[..ver_sep];
          let rest = &package[ver_sep + 1..];
          if let Some(sub_slash) = rest.find('/') {
            let _ver = &rest[..sub_slash];
            (name, Some(&rest[sub_slash + 1..]))
          } else {
            (name, None)
          }
        } else {
          let first_slash = package.find('/').unwrap();
          (&package[..first_slash], Some(&package[first_slash + 1..]))
        }
      }
    } else {
      // plain package or package@version
      if let Some(ver_sep) = package.find('@') {
        if package.starts_with('@') {
          (package, None)
        } else {
          (&package[..ver_sep], None)
        }
      } else {
        (package, None)
      }
    };

    let base_resolved = self.resolve_node_modules(package_name, &self.referrer_search_dir(referrer))?;
    if let Some(sub) = subpath {
      if let Ok(base_path) = base_resolved.to_file_path() {
        if let Some(parent) = base_path.parent() {
          let sub_path = parent.join(sub);
          return self.resolve_file_path(&sub_path);
        }
      }
    }
    Ok(base_resolved)
  }
}

impl ModuleLoader for KlyronModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> ModuleResolveResponse {
    // 0. Try import map first
    let referrer_str = if referrer.starts_with("file://") {
      Some(referrer)
    } else {
      None
    };
    if let Some(mapped) = self.import_map.resolve(specifier, referrer_str) {
      if mapped.starts_with("file://") {
        return Ok(ModuleSpecifier::parse(&mapped).unwrap());
      }
      if mapped.starts_with("https://") || mapped.starts_with("http://") {
        return Ok(ModuleSpecifier::parse(&mapped).unwrap());
      }
       if mapped.starts_with("npm:") {
         return self.resolve_npm(&mapped, referrer);
       }
      if mapped.starts_with("node:") {
        let builtin = mapped.strip_prefix("node:").unwrap_or(&mapped);
        for (name, url) in NODE_BUILTINS.iter() {
          if *name == builtin {
            return Ok(ModuleSpecifier::parse(url).unwrap());
          }
        }
        return Err(JsErrorBox::generic(format!("Unknown node:builtin: {builtin}")));
      }
      // Fallback: treat mapped as relative path from project root
      if let Ok(path) = std::env::current_dir() {
        let candidate = path.join(&mapped);
        return self.resolve_file_path(&candidate);
      }
    }

    // 1. node: builtins -> ext:klyron_node/*.js
    if let Some(builtin) = specifier.strip_prefix("node:") {
      for (name, url) in NODE_BUILTINS.iter() {
        if *name == builtin {
          return Ok(ModuleSpecifier::parse(url).unwrap());
        }
      }
      return Err(JsErrorBox::generic(format!("Unknown node:builtin: {builtin}")));
    }

    // 2. npm: specifiers
    if specifier.starts_with("npm:") {
      return self.resolve_npm(specifier, referrer);
    }

    // 3. ext: / klyron: -> deno_core extension modules
    if specifier.starts_with("ext:") || specifier.starts_with("klyron:") {
      return Ok(ModuleSpecifier::parse(specifier).unwrap());
    }

    // 4. file:// protocol
    if specifier.starts_with("file://") {
      let path = specifier.strip_prefix("file://").unwrap_or(specifier);
      let path_buf = PathBuf::from(path);
      return self.resolve_file_path(&path_buf);
    }

    // 5. data: and blob: URLs
    if specifier.starts_with("data:") || specifier.starts_with("blob:") {
      return Ok(ModuleSpecifier::parse(specifier).unwrap());
    }

    // 6. Relative imports
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

    // 7. Absolute paths
    if specifier.starts_with('/') {
      return self.resolve_file_path(&PathBuf::from(specifier));
    }

    // 8. Bare specifier -> node_modules
    self.resolve_node_modules(specifier, &self.referrer_search_dir(referrer))
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

    // npm: / https: / http: remote fetching
    if url.starts_with("https://") || url.starts_with("http://") || url.starts_with("npm:") {
      if let Ok(path) = std::env::current_dir() {
        let cache_dir = path.join(".klyron").join("cache").join("npm");
        let cache_key = url.replace(&['/', ':', '@', '#'][..], "_");
        let cache_file = cache_dir.join(&cache_key);
        if cache_file.exists() {
          if let Ok(content) = std::fs::read_to_string(&cache_file) {
            let cached = CachedModule {
              code: content.clone(),
              module_type: ModuleType::JavaScript,
            };
            self.cache_set(url.clone(), cached);
            let code = ModuleSourceCode::String(content.into());
            let src = ModuleSource::new(ModuleType::JavaScript, code, module_specifier, None);
            return ModuleLoadResponse::Sync(Ok(src));
          }
        }
      }
      // For now, return error for remote modules without cache
      return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
        "Remote module not cached: {url}. Run `klyron cache add {url}` first."
      ))));
    }

    // File system (including file:// URIs)
    let path = if url.starts_with("file://") {
      match module_specifier.to_file_path() {
        Ok(p) => p,
        Err(_) => {
          return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
            "Invalid file URI: {url}"
          ))));
        }
      }
    } else {
      match module_specifier.to_file_path() {
        Ok(p) => p,
        Err(_) => {
          return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
            "Module not found: {url}"
          ))));
        }
      }
    };

    if path.exists() {
      let path_str = path.to_string_lossy();
      if let Err(e) = self.permissions.check_read(&path_str) {
        return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(e)));
      }

      match std::fs::read_to_string(&path) {
        Ok(content) => {
          let is_ts = path
            .extension()
            .map(|e| e == "ts" || e == "tsx" || e == "mts" || e == "cts")
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

    ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!(
      "Module not found: {url}"
    ))))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::permissions::PermissionSet;
  use deno_core::ResolutionKind;

  fn loader() -> KlyronModuleLoader {
    KlyronModuleLoader::new(
      Arc::new(Permissions::new(PermissionSet::default())),
      None,
    )
  }

  fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
  }

  fn resolved_path(l: &KlyronModuleLoader, spec: &str, referrer: &str) -> String {
    l.resolve(spec, referrer, ResolutionKind::DynamicImport)
      .unwrap()
      .to_string()
  }

  #[test]
  fn test_resolve_package_main() {
    let tmp = std::env::temp_dir().join(format!("klyron_ml_main_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let referrer = format!("file://{}/app.js", tmp.display());
    let nm = tmp.join("node_modules").join("demo");
    write(&nm.join("package.json"), r#"{"name":"demo","main":"lib/entry.js"}"#);
    write(&nm.join("lib/entry.js"), "export const x = 1;");

    let l = loader();
    let got = resolved_path(&l, "demo", &referrer);
    assert!(got.ends_with("/node_modules/demo/lib/entry.js"), "got: {got}");
    let _ = std::fs::remove_dir_all(&tmp);
  }

  #[test]
  fn test_resolve_package_exports_string() {
    let tmp = std::env::temp_dir().join(format!("klyron_ml_exp_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let referrer = format!("file://{}/app.js", tmp.display());
    let nm = tmp.join("node_modules").join("demo");
    write(&nm.join("package.json"), r#"{"name":"demo","exports":"./dist/index.mjs"}"#);
    write(&nm.join("dist/index.mjs"), "export const x = 1;");

    let l = loader();
    let got = resolved_path(&l, "demo", &referrer);
    assert!(got.ends_with("/node_modules/demo/dist/index.mjs"), "got: {got}");
    let _ = std::fs::remove_dir_all(&tmp);
  }

  #[test]
  fn test_resolve_package_exports_condition_map() {
    let tmp = std::env::temp_dir().join(format!("klyron_ml_expc_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let referrer = format!("file://{}/app.js", tmp.display());
    let nm = tmp.join("node_modules").join("demo");
    write(
      &nm.join("package.json"),
      r#"{"name":"demo","exports":{".":{"import":"./esm.mjs","require":"./cjs.js"}}}"#,
    );
    write(&nm.join("esm.mjs"), "export const x = 1;");
    write(&nm.join("cjs.js"), "module.exports = 1;");

    let l = loader();
    let got = resolved_path(&l, "demo", &referrer);
    assert!(got.ends_with("/node_modules/demo/esm.mjs"), "got: {got}");
    let _ = std::fs::remove_dir_all(&tmp);
  }

  #[test]
  fn test_resolve_package_fallback_index() {
    let tmp = std::env::temp_dir().join(format!("klyron_ml_idx_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let referrer = format!("file://{}/app.js", tmp.display());
    let nm = tmp.join("node_modules").join("demo");
    write(&nm.join("package.json"), r#"{"name":"demo"}"#);
    write(&nm.join("index.js"), "export const x = 1;");

    let l = loader();
    let got = resolved_path(&l, "demo", &referrer);
    assert!(got.ends_with("/node_modules/demo/index.js"), "got: {got}");
    let _ = std::fs::remove_dir_all(&tmp);
  }
}
