use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleFormat { CommonJS, ESM, Unknown }

#[derive(Debug, Clone)]
pub struct ModuleResolution {
    pub resolved_path: PathBuf,
    pub format: ModuleFormat,
    pub kind: ModuleKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind { Unknown, JavaScript, TypeScript, Json, NodeBuiltin }
impl Default for ModuleKind { fn default() -> Self { Self::Unknown } }

pub struct ModuleResolver {
    pub imports_map: HashMap<String, String>,
}
impl Default for ModuleResolver { fn default() -> Self { Self::new() } }
impl ModuleResolver {
    pub fn new() -> Self { Self { imports_map: HashMap::new() } }
    pub fn detect_format(path: &Path) -> ModuleFormat { let n = path.file_name().unwrap_or_default().to_string_lossy(); if n.ends_with(".mjs")||n.ends_with(".mts") { ModuleFormat::ESM } else { ModuleFormat::CommonJS } }
    pub fn resolve(&self, specifier: &str, base: &Path) -> ModuleResolution { let _ = (specifier, base); ModuleResolution { resolved_path: PathBuf::from(specifier), format: ModuleFormat::Unknown, kind: ModuleKind::Unknown } }
    pub fn resolve_import_map(&self, specifier: &str) -> Option<String> { for (k, v) in &self.imports_map { if specifier.starts_with(k.trim_end_matches('*')) { if k.ends_with('*') { return Some(v.replace('*', specifier.strip_prefix(k.trim_end_matches('*'))?)); } return Some(v.clone()); } } None }
}

pub fn cjs_to_esm(source: &str) -> String { source.replace("module.exports = ", "export default ").replace("exports.", "export const ").replace("require(", "import(") }
pub fn esm_to_cjs(source: &str) -> String { source.replace("export default ", "module.exports = ").replace("export const ", "exports.").replace("import(", "require(") }
