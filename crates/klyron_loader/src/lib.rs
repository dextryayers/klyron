use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleFormat {
    CommonJS,
    ESM,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ModuleResolution {
    pub resolved_path: PathBuf,
    pub format: ModuleFormat,
    pub kind: ModuleKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    Unknown,
    JavaScript,
    TypeScript,
    Json,
    NodeBuiltin,
}

impl Default for ModuleKind { fn default() -> Self { Self::Unknown } }

pub struct ModuleResolver {
    pub imports_map: HashMap<String, String>,
}

impl Default for ModuleResolver { fn default() -> Self { Self::new() } }

impl ModuleResolver {
    pub fn new() -> Self { Self { imports_map: HashMap::new() } }

    pub fn detect_format(path: &Path) -> ModuleFormat {
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        if filename.ends_with(".mjs") || filename.ends_with(".mts") {
            return ModuleFormat::ESM;
        }
        if filename.ends_with(".cjs") || filename.ends_with(".cts") {
            return ModuleFormat::CommonJS;
        }
        if filename.ends_with(".js") || filename.ends_with(".ts") {
            if let Some(dir) = path.parent() {
                let pkg_json = dir.join("package.json");
                if pkg_json.exists() {
                    if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(module_type) = pkg.get("type").and_then(|t| t.as_str()) {
                                return if module_type == "module" { ModuleFormat::ESM } else { ModuleFormat::CommonJS };
                            }
                        }
                    }
                }
            }
        }
        ModuleFormat::CommonJS
    }

    pub fn resolve_import(specifier: &str, base: &Path) -> PathBuf {
        if specifier.starts_with(".") || specifier.starts_with("/") {
            let candidate = base.parent().unwrap_or(base).join(specifier);
            Self::resolve_file(&candidate).unwrap_or(candidate)
        } else if specifier.starts_with("node:") {
            PathBuf::from(specifier)
        } else if specifier.starts_with("npm:") {
            PathBuf::from(specifier)
        } else {
            let node_modules = Self::find_node_modules(base, specifier);
            node_modules.unwrap_or_else(|| PathBuf::from(specifier))
        }
    }

    fn resolve_file(path: &Path) -> Option<PathBuf> {
        if path.exists() { return Some(path.to_path_buf()); }
        for ext in &[".js", ".ts", ".jsx", ".tsx", ".json", ".mjs", ".cjs", ".mts", ".cts"] {
            let with_ext = path.with_extension(ext.trim_start_matches('.'));
            if with_ext.exists() { return Some(with_ext); }
        }
        let candidates = [path.join("index.js"), path.join("index.ts"), path.join("index.json"), path.join("index.mjs")];
        for c in &candidates {
            if c.exists() { return Some(c.clone()); }
        }
        None
    }

    fn find_node_modules(base: &Path, specifier: &str) -> Option<PathBuf> {
        let mut dir = Some(base);
        while let Some(d) = dir {
            let nm = d.join("node_modules").join(specifier);
            if let Some(resolved) = Self::resolve_file(&nm) {
                return Some(resolved);
            }
            if let Some(pkg_main) = Self::resolve_package_json(&nm) {
                return Some(pkg_main);
            }
            dir = d.parent();
        }
        None
    }

    fn resolve_package_json(pkg_dir: &Path) -> Option<PathBuf> {
        let pkg_json = pkg_dir.join("package.json");
        if pkg_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(exports) = pkg.get("exports") {
                        if let Some(import) = exports.get("import").or_else(|| exports.get("default")) {
                            if let Some(s) = import.as_str() {
                                let entry = pkg_dir.join(s);
                                if entry.exists() { return Some(entry); }
                            }
                        }
                        if let Some(require) = exports.get("require") {
                            if let Some(s) = require.as_str() {
                                let entry = pkg_dir.join(s);
                                if entry.exists() { return Some(entry); }
                            }
                        }
                    }
                    if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
                        let entry = pkg_dir.join(main);
                        if let Some(resolved) = Self::resolve_file(&entry) {
                            return Some(resolved);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn resolve_import_map(&self, specifier: &str) -> Option<String> {
        for (key, value) in &self.imports_map {
            if specifier.starts_with(key.trim_end_matches('*')) {
                if key.ends_with('*') {
                    let suffix = specifier.strip_prefix(key.trim_end_matches('*'))?;
                    return Some(value.replace('*', suffix));
                }
                if specifier == key || key.is_empty() {
                    return Some(value.clone());
                }
            }
        }
        None
    }

    pub fn resolve(&self, specifier: &str, base: &Path) -> ModuleResolution {
        if let Some(mapped) = self.resolve_import_map(specifier) {
            let path = PathBuf::from(&mapped);
            let format = Self::detect_format(&path);
            return ModuleResolution { resolved_path: path, format, kind: ModuleKind::JavaScript };
        }
        let path = Self::resolve_import(specifier, base);
        let format = Self::detect_format(&path);
        let kind = if path.extension().map_or(false, |e| e == "ts" || e == "tsx") { ModuleKind::TypeScript }
            else if path.extension().map_or(false, |e| e == "json") { ModuleKind::Json }
            else { ModuleKind::JavaScript };
        ModuleResolution { resolved_path: path, format, kind }
    }
}

pub fn cjs_to_esm(source: &str) -> String {
    let mut out = source.to_string();
    out = out.replace("module.exports = ", "export default ");
    out = out.replace("exports.", "export const ");
    out = out.replace("require(", "import(");
    out
}

pub fn esm_to_cjs(source: &str) -> String {
    let mut out = source.to_string();
    out = out.replace("export default ", "module.exports = ");
    out = out.replace("export const ", "exports.");
    out = out.replace("import(", "require(");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_detect_format_esm() {
        let path = Path::new("/test/file.mjs");
        assert_eq!(ModuleResolver::detect_format(path), ModuleFormat::ESM);
    }

    #[test]
    fn test_detect_format_cjs() {
        let path = Path::new("/test/file.cjs");
        assert_eq!(ModuleResolver::detect_format(path), ModuleFormat::CommonJS);
    }

    #[test]
    fn test_resolve_import_map() {
        let mut resolver = ModuleResolver::new();
        resolver.imports_map.insert("preact".to_string(), "https://esm.sh/preact".to_string());
        resolver.imports_map.insert("std/*".to_string(), "https://deno.land/std@0.224.0/*".to_string());
        assert_eq!(resolver.resolve_import_map("preact"), Some("https://esm.sh/preact".to_string()));
        assert_eq!(resolver.resolve_import_map("std/fs/mod.ts"), Some("https://deno.land/std@0.224.0/fs/mod.ts".to_string()));
    }

    #[test]
    fn test_cjs_to_esm() {
        let cjs = "module.exports = { foo: 1 }";
        let esm = cjs_to_esm(cjs);
        assert_eq!(esm, "export default { foo: 1 }");
    }

    #[test]
    fn test_esm_to_cjs() {
        let esm = "export default { foo: 1 }";
        let cjs = esm_to_cjs(esm);
        assert_eq!(cjs, "module.exports = { foo: 1 }");
    }
}
