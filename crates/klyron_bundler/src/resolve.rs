use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::Regex;

use crate::{ExportInfo, ModuleExports};

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub entries: Vec<PathBuf>,
    pub modules: HashMap<PathBuf, ModuleNode>,
    pub circular_deps: Vec<Vec<PathBuf>>,
}

#[derive(Debug, Clone)]
pub struct ModuleNode {
    pub path: PathBuf,
    pub dependencies: Vec<PathBuf>,
    pub dependents: Vec<PathBuf>,
    pub dynamic_imports: Vec<String>,
    pub is_entry: bool,
    pub is_external: bool,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            modules: HashMap::new(),
            circular_deps: Vec::new(),
        }
    }

    pub fn get_entry_modules(&self) -> Vec<&ModuleNode> {
        self.modules.values().filter(|m| m.is_entry).collect()
    }

    pub fn get_transitive_deps(&self, path: &Path) -> Vec<&ModuleNode> {
        let mut deps = Vec::new();
        let mut visited = HashSet::new();
        self.collect_deps(path, &mut deps, &mut visited);
        deps
    }

    fn collect_deps<'a>(&'a self, path: &Path, deps: &mut Vec<&'a ModuleNode>, visited: &mut HashSet<PathBuf>) {
        if !visited.insert(path.to_path_buf()) {
            return;
        }
        if let Some(_node) = self.modules.get(path) {
            for dep in &_node.dependencies {
                if let Some(dep_node) = self.modules.get(dep) {
                    deps.push(dep_node);
                    self.collect_deps(dep, deps, visited);
                }
            }
        }
    }

    pub fn find_unused_exports(&self, entry: &Path) -> HashSet<String> {
        let mut used = HashSet::new();
        let mut visited = HashSet::new();
        self.collect_used_exports(entry, &mut used, &mut visited);
        used
    }

    fn collect_used_exports(&self, path: &Path, used: &mut HashSet<String>, visited: &mut HashSet<PathBuf>) {
        if !visited.insert(path.to_path_buf()) {
            return;
        }
        if let Some(_node) = self.modules.get(path) {
            if let Ok(content) = std::fs::read_to_string(path) {
                let import_re = Regex::new(r#"import\s+\{?\s*([^}]+)\s*\}?\s*from\s+["']([^"']+)["']"#).unwrap();
                for line in content.lines() {
                    if let Some(caps) = import_re.captures(line.trim()) {
                        let imports = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        let spec = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                        for name in imports.split(',').map(|s| s.trim()) {
                            let name = name.split(" as ").next().unwrap_or(name).trim();
                            used.insert(name.to_string());
                        }
                        if spec.starts_with('.') {
                            let resolved = resolve_specifier(path, spec);
                            if let Some(resolved) = resolved {
                                self.collect_used_exports(&resolved, used, visited);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn has_circular_dependency(&self) -> bool {
        !self.circular_deps.is_empty()
    }
}

pub fn resolve_deps(entry: &Path) -> Result<DependencyGraph> {
    let mut graph = DependencyGraph::new();
    let mut visited = HashSet::new();
    let mut stack: Vec<PathBuf> = Vec::new();

    graph.entries.push(entry.to_path_buf());
    resolve_module(entry, &mut graph, &mut visited, &mut stack)?;

    Ok(graph)
}

fn resolve_module(
    file: &Path,
    graph: &mut DependencyGraph,
    visited: &mut HashSet<PathBuf>,
    stack: &mut Vec<PathBuf>,
) -> Result<()> {
    if !file.exists() {
        return Ok(());
    }

    let file_buf = file.to_path_buf();
    if stack.contains(&file_buf) {
        let cycle_start = stack.iter().position(|p| p == &file_buf).unwrap();
        graph.circular_deps.push(stack[cycle_start..].to_vec());
        return Ok(());
    }

    if !visited.insert(file_buf.clone()) {
        return Ok(());
    }

    stack.push(file_buf);

    let content = std::fs::read_to_string(file)?;
    let mut deps = Vec::new();
    let mut dynamic_imports = Vec::new();

    let import_re = Regex::new(r#"(?:import|require)\s*\(?\s*["']([^"']+)["']\s*\)?"#).unwrap();
    let dynamic_re = Regex::new(r#"import\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(caps) = dynamic_re.captures(trimmed) {
            let spec = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            dynamic_imports.push(spec.clone());
            if spec.starts_with('.') {
                if let Some(resolved) = resolve_specifier(file, &spec) {
                    deps.push(resolved.clone());
                    resolve_module(&resolved, graph, visited, stack)?;
                }
            }
            continue;
        }

        if let Some(caps) = import_re.captures(trimmed) {
            let spec = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            if spec.starts_with('.') || spec.starts_with('/') {
                if let Some(resolved) = resolve_specifier(file, spec) {
                    deps.push(resolved.clone());
                    resolve_module(&resolved, graph, visited, stack)?;
                }
            }
        }
    }

    let node = ModuleNode {
        path: file.to_path_buf(),
        dependencies: deps.clone(),
        dependents: Vec::new(),
        dynamic_imports,
        is_entry: graph.entries.contains(&file.to_path_buf()),
        is_external: false,
    };

    graph.modules.insert(file.to_path_buf(), node);

    for dep in &deps {
        if let Some(dep_node) = graph.modules.get_mut(dep) {
            dep_node.dependents.push(file.to_path_buf());
        }
    }

    stack.pop();
    Ok(())
}

pub fn resolve_specifier(source: &Path, specifier: &str) -> Option<PathBuf> {
    let parent = source.parent().unwrap_or(Path::new("."));
    let candidate = if specifier.starts_with('/') {
        PathBuf::from(specifier.trim_start_matches('/'))
    } else {
        parent.join(specifier)
    };

    let extensions = ["", ".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".json", ".node"];
    let index_files = ["index.js", "index.ts", "index.jsx", "index.tsx", "index.mjs", "index.cjs"];

    if candidate.is_file() {
        return Some(candidate);
    }

    for ext in &extensions {
        if !ext.is_empty() {
            let with_ext = candidate.with_extension(ext.trim_start_matches('.'));
            if with_ext.is_file() {
                return Some(with_ext);
            }
        }
    }

    if candidate.is_dir() {
        for index in &index_files {
            let idx_path = candidate.join(index);
            if idx_path.is_file() {
                return Some(idx_path);
            }
        }
    }

    None
}

pub fn analyze_exports(entry: &Path) -> Result<Vec<ModuleExports>> {
    let mut all_exports = Vec::new();
    let mut visited = HashSet::new();
    analyze_exports_recursive(entry, &mut all_exports, &mut visited)?;
    Ok(all_exports)
}

fn analyze_exports_recursive(
    file: &Path,
    all_exports: &mut Vec<ModuleExports>,
    visited: &mut HashSet<PathBuf>,
) -> Result<()> {
    if !file.exists() || !visited.insert(file.to_path_buf()) {
        return Ok(());
    }

    let content = std::fs::read_to_string(file)?;
    let mut exports = Vec::new();
    let mut re_exports = Vec::new();

    let export_named_re = Regex::new(r#"export\s+(?:const|function|class|let|var|type|interface)\s+(\w+)"#).unwrap();
    let export_default_re = Regex::new(r#"export\s+default\s+(?:function|class|const)?\s*(\w*)"#).unwrap();
    let export_re_export_re = Regex::new(r#"export\s+\{?\s*([^}]+)\s*\}?\s*from\s+["']([^"']+)["']"#).unwrap();
    let import_re = Regex::new(r#"import\s+\{?\s*([^}]+)\s*\}?\s*from\s+["']([^"']+)["']"#).unwrap();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(caps) = export_named_re.captures(trimmed) {
            exports.push(ExportInfo {
                name: caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
                local_name: caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
                is_default: false,
                used: true,
            });
        }

        if let Some(caps) = export_default_re.captures(trimmed) {
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or("default");
            let name = if name.is_empty() { "default" } else { name };
            exports.push(ExportInfo {
                name: name.to_string(),
                local_name: name.to_string(),
                is_default: true,
                used: true,
            });
        }

        if let Some(caps) = export_re_export_re.captures(trimmed) {
            let spec = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            re_exports.push(spec.clone());
            if spec.starts_with('.') {
                if let Some(resolved) = resolve_specifier(file, &spec) {
                    analyze_exports_recursive(&resolved, all_exports, visited)?;
                }
            }
        }

        if let Some(caps) = import_re.captures(trimmed) {
            let imports_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let spec = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            if spec.starts_with('.') {
                if let Some(resolved) = resolve_specifier(file, spec) {
                    analyze_exports_recursive(&resolved, all_exports, visited)?;
                    for import_name in imports_str.split(',').map(|s| s.trim().trim_start_matches("type ")) {
                        let import_name = import_name.split(" as ").next().unwrap_or(import_name).trim();
                        for module_exports in all_exports.iter_mut() {
                            for export in module_exports.exports.iter_mut() {
                                if export.name == import_name {
                                    export.used = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    all_exports.push(ModuleExports {
        file: file.to_string_lossy().to_string(),
        exports,
        re_exports,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("klyron_test_resolve_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_resolve_specifier_with_ext() {
        let dir = temp_dir("resolve_ext");
        fs::write(dir.join("foo.js"), "export const x = 1;").unwrap();
        let resolved = resolve_specifier(&dir.join("index.js"), "./foo");
        assert!(resolved.is_some());
        assert!(resolved.unwrap().ends_with("foo.js"));
    }

    #[test]
    fn test_resolve_specifier_index() {
        let dir = temp_dir("resolve_index");
        fs::create_dir_all(dir.join("bar")).unwrap();
        fs::write(dir.join("bar/index.js"), "export const y = 2;").unwrap();
        let resolved = resolve_specifier(&dir.join("index.js"), "./bar");
        assert!(resolved.is_some());
        assert!(resolved.unwrap().ends_with("bar/index.js"));
    }

    #[test]
    fn test_resolve_specifier_non_existent() {
        let dir = temp_dir("resolve_missing");
        let resolved = resolve_specifier(&dir.join("index.js"), "./nonexistent");
        assert!(resolved.is_none());
    }

    #[test]
    fn test_dependency_graph() {
        let dir = temp_dir("dep_graph");
        fs::write(dir.join("entry.js"), "import { foo } from './foo';\nimport('./lazy').then(m => m.default());").unwrap();
        fs::write(dir.join("foo.js"), "export const foo = 1;").unwrap();
        let graph = resolve_deps(&dir.join("entry.js")).unwrap();
        assert!(graph.modules.contains_key(&dir.join("entry.js")));
        assert!(graph.modules.contains_key(&dir.join("foo.js")));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let dir = temp_dir("circular");
        fs::write(dir.join("a.js"), "import { b } from './b'; export const a = 1;").unwrap();
        fs::write(dir.join("b.js"), "import { a } from './a'; export const b = 2;").unwrap();
        let graph = resolve_deps(&dir.join("a.js")).unwrap();
        assert!(graph.has_circular_dependency());
    }

    #[test]
    fn test_analyze_exports() {
        let dir = temp_dir("analyze_exports");
        fs::write(dir.join("index.js"), "export const foo = 1;\nexport function bar() {}\nexport default class Baz {}").unwrap();
        let modules = analyze_exports(&dir.join("index.js")).unwrap();
        assert!(!modules.is_empty());
        let me = &modules[0];
        assert!(me.exports.iter().any(|e| e.name == "foo"));
        assert!(me.exports.iter().any(|e| e.name == "bar"));
        assert!(me.exports.iter().any(|e| e.is_default));
    }

    #[test]
    fn test_unused_exports() {
        let dir = temp_dir("unused_exports");
        fs::write(dir.join("entry.js"), "import { used } from './lib'; console.log(used);").unwrap();
        fs::write(dir.join("lib.js"), "export const used = 1;\nexport const unused = 2;").unwrap();
        let graph = resolve_deps(&dir.join("entry.js")).unwrap();
        let _unused = graph.find_unused_exports(&dir.join("entry.js"));
        let modules = analyze_exports(&dir.join("entry.js")).unwrap();
        let lib_exports = modules.iter().find(|m| m.file.contains("lib.js"));
        assert!(lib_exports.is_some());
    }
}
