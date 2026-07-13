use std::{
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
}

impl KlyronModuleLoader {
  pub fn new(permissions: Arc<Permissions>, transpiler: Option<Transpiler>) -> Self {
    Self { permissions, transpiler, cache: Arc::new(Mutex::new(Vec::new())) }
  }

  fn cache_get(&self, url: &str) -> Option<CachedModule> {
    self.cache.lock().ok().and_then(|c| c.iter().find(|(k, _)| k == url).map(|(_, s)| CachedModule { code: s.code.clone(), module_type: s.module_type.clone() }))
  }

  fn cache_set(&self, url: String, module: CachedModule) {
    if let Ok(mut c) = self.cache.lock() { c.push((url, module)); }
  }

  fn resolve_file_path(&self, path: &Path) -> ModuleResolveResponse {
    let extensions = &[".js", ".ts", ".tsx", ".jsx", ".json", ".mjs", ".cjs"][..];
    if path.exists() && path.is_file() {
      return self.to_specifier(path);
    }
    for ext in extensions {
      let with_ext = path.with_extension(ext.trim_start_matches('.'));
      if with_ext.exists() { return self.to_specifier(&with_ext); }
    }
    for ext in extensions {
      let index = path.join(format!("index{ext}"));
      if index.exists() { return self.to_specifier(&index); }
    }
    Err(JsErrorBox::generic(format!("Module not found: {}", path.display())))
  }

  fn to_specifier(&self, path: &Path) -> ModuleResolveResponse {
    Ok(ModuleSpecifier::parse(&format!("file://{}", path.canonicalize().unwrap_or_else(|_| path.to_path_buf()).display())).unwrap())
  }

  fn resolve_node_modules(&self, name: &str, referrer_dir: &Path) -> ModuleResolveResponse {
    let mut dir = Some(referrer_dir);
    while let Some(current) = dir {
      let nm = current.join("node_modules").join(name);
      if nm.exists() { return self.resolve_file_path(&nm); }
      dir = current.parent();
    }
    Err(JsErrorBox::generic(format!("Cannot find module '{name}'")))
  }
}

impl ModuleLoader for KlyronModuleLoader {
  fn resolve(&self, specifier: &str, referrer: &str, _kind: ResolutionKind) -> ModuleResolveResponse {
    // node: builtins → ext:klyron_node/*.js
    if let Some(builtin) = specifier.strip_prefix("node:") {
      for (name, url) in NODE_BUILTINS.iter() {
        if *name == builtin {
          return Ok(ModuleSpecifier::parse(url).unwrap());
        }
      }
      return Err(JsErrorBox::generic(format!("Unknown node:builtin: {builtin}")));
    }

    // ext: / klyron: → deno_core extension modules
    if specifier.starts_with("ext:") || specifier.starts_with("klyron:") {
      return Ok(ModuleSpecifier::parse(specifier).unwrap());
    }

    // file:// protocol
    if specifier.starts_with("file://") {
      return Ok(ModuleSpecifier::parse(specifier).unwrap());
    }

    // Relative imports
    if specifier.starts_with("./") || specifier.starts_with("../") {
      if referrer.starts_with("ext:") || referrer.starts_with("klyron:") {
        let base_url = ModuleSpecifier::parse(referrer).unwrap();
        return Ok(base_url.join(specifier).unwrap());
      }
      let base_path = if referrer.starts_with("file://") {
        ModuleSpecifier::parse(referrer).ok().and_then(|s| s.to_file_path().ok()).unwrap_or_else(|| std::env::current_dir().unwrap())
      } else {
        std::env::current_dir().unwrap()
      };
      let base_dir = if base_path.is_file() { base_path.parent().unwrap().to_path_buf() } else { base_path };
      return self.resolve_file_path(&base_dir.join(specifier));
    }

    if specifier.starts_with('/') {
      return self.resolve_file_path(&PathBuf::from(specifier));
    }

    // Bare specifier → node_modules
    self.resolve_node_modules(specifier, &std::env::current_dir().unwrap())
  }

  fn load(&self, module_specifier: &ModuleSpecifier, _maybe_referrer: Option<&ModuleLoadReferrer>, _options: ModuleLoadOptions) -> ModuleLoadResponse {
    let url = module_specifier.to_string();

    // ext: modules are handled by deno_core internally
    if url.starts_with("ext:") || url.starts_with("klyron:") {
      return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!("ext: modules handled by deno_core: {url}"))));
    }

    if let Some(cached) = self.cache_get(&url) {
      let code = ModuleSourceCode::String(cached.code.into());
      let src = ModuleSource::new(cached.module_type, code, module_specifier, None);
      return ModuleLoadResponse::Sync(Ok(src));
    }

    if let Ok(path) = module_specifier.to_file_path() {
      if path.exists() {
        // Check read permission before loading
        let path_str = path.to_string_lossy();
        if let Err(e) = self.permissions.check_read(&path_str) {
          return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(e)));
        }

        match std::fs::read_to_string(&path) {
          Ok(content) => {
            let is_ts = path.extension().map(|e| e == "ts" || e == "tsx").unwrap_or(false);
            let module_type = if path.extension().map(|e| e == "json").unwrap_or(false) {
              ModuleType::Json
            } else {
              ModuleType::JavaScript
            };
            let code = if is_ts {
              if let Some(ref t) = self.transpiler { t.transpile(&content).unwrap_or(content) } else { content }
            } else { content };
            let cached = CachedModule { code: code.clone(), module_type: module_type.clone() };
            self.cache_set(url, cached);
            let src = ModuleSource::new(module_type, ModuleSourceCode::String(code.into()), module_specifier, None);
            return ModuleLoadResponse::Sync(Ok(src));
          }
          Err(e) => return ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!("Read error {path:?}: {e}")))),
        }
      }
    }

    ModuleLoadResponse::Sync(Err(JsErrorBox::generic(format!("Module not found: {url}"))))
  }
}
