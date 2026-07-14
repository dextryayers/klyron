pub mod compat;
pub mod polyfill;

pub use compat::NodeCompat;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct NodeModule {
    pub id: String,
    pub filename: PathBuf,
    pub dirname: PathBuf,
    pub exports: HashMap<String, serde_json::Value>,
    pub children: Vec<NodeModule>,
}

pub struct RequireContext {
    pub current_file: PathBuf,
    pub paths: Vec<PathBuf>,
}

impl RequireContext {
    pub fn new(current_file: &Path) -> Self {
        let dir = current_file.parent().unwrap_or(Path::new("."));
        Self {
            current_file: current_file.to_path_buf(),
            paths: {
                let mut pths = vec![dir.join("node_modules")];
                pths.extend(dir.ancestors().skip(1).map(|p| p.join("node_modules")));
                pths
            },
        }
    }

    pub fn resolve(&self, specifier: &str) -> Option<PathBuf> {
        if specifier.starts_with(".") || specifier.starts_with("/") {
            let base = self.current_file.parent().unwrap_or(Path::new("."));
            let candidate = base.join(specifier);
            return resolve_file(&candidate);
        }
        for path in &self.paths {
            let candidate = path.join(specifier);
            if let Some(resolved) = resolve_file(&candidate) {
                return Some(resolved);
            }
            let pkg_json = candidate.join("package.json");
            if pkg_json.exists() {
                if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                    if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
                            let entry = candidate.join(main);
                            if let Some(resolved) = resolve_file(&entry) {
                                return Some(resolved);
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

fn resolve_file(path: &Path) -> Option<PathBuf> {
    if path.is_file() { return Some(path.to_path_buf()); }
    for ext in &[".js", ".json", ".node", ".mjs", ".cjs"] {
        let with_ext = path.with_extension(ext.trim_start_matches('.'));
        if with_ext.is_file() { return Some(with_ext); }
    }
    let index_js = path.join("index.js");
    if index_js.is_file() { return Some(index_js); }
    let index_json = path.join("index.json");
    if index_json.is_file() { return Some(index_json); }
    None
}

pub fn get_dirname(path: &Path) -> PathBuf {
    path.parent().unwrap_or(Path::new(".")).to_path_buf()
}

pub fn get_filename(path: &Path) -> PathBuf {
    path.to_path_buf()
}
