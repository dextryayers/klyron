use std::path::{Path, PathBuf};

pub struct PathUtil;

impl PathUtil {
    pub fn join(base: &Path, segments: &[&str]) -> PathBuf {
        let mut p = base.to_path_buf();
        for s in segments {
            p = p.join(s);
        }
        p
    }

    pub fn extension(p: &Path) -> &str {
        p.extension().and_then(|e| e.to_str()).unwrap_or("")
    }

    pub fn file_stem(p: &Path) -> &str {
        p.file_stem().and_then(|s| s.to_str()).unwrap_or("")
    }

    pub fn normalize(p: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in p.components() {
            match component {
                std::path::Component::Normal(c) => components.push(c),
                std::path::Component::ParentDir => {
                    components.pop();
                }
                _ => {}
            }
        }
        let mut result = PathBuf::new();
        for c in components {
            result = result.join(c);
        }
        result
    }

    pub fn is_js_like(p: &Path) -> bool {
        matches!(Self::extension(p), "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs")
    }

    pub fn to_unix(p: &Path) -> String {
        p.to_string_lossy().replace('\\', "/")
    }

    pub fn find_up(start: &Path, filename: &str) -> Option<PathBuf> {
        let mut current = Some(start.to_path_buf());
        while let Some(dir) = current {
            let candidate = dir.join(filename);
            if candidate.exists() {
                return Some(candidate);
            }
            current = dir.parent().map(|p| p.to_path_buf());
        }
        None
    }

    pub fn find_all_up(start: &Path, filename: &str) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let mut current = Some(start.to_path_buf());
        while let Some(dir) = current {
            let candidate = dir.join(filename);
            if candidate.exists() {
                results.push(candidate);
            }
            current = dir.parent().map(|p| p.to_path_buf());
        }
        results
    }

    pub fn relative_components(from: &Path, to: &Path) -> PathBuf {
        let mut result = PathBuf::new();
        let from_components: Vec<_> = from.components().collect();
        let to_components: Vec<_> = to.components().collect();
        let common_len = from_components
            .iter()
            .zip(to_components.iter())
            .take_while(|(a, b)| a == b)
            .count();
        for _ in common_len..from_components.len() {
            result.push("..");
        }
        for c in &to_components[common_len..] {
            result.push(c.as_os_str());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_util_extension() {
        assert_eq!(PathUtil::extension(Path::new("test.js")), "js");
        assert_eq!(PathUtil::extension(Path::new("test.tsx")), "tsx");
    }

    #[test]
    fn test_path_util_is_js_like() {
        assert!(PathUtil::is_js_like(Path::new("test.tsx")));
        assert!(!PathUtil::is_js_like(Path::new("test.py")));
    }

    #[test]
    fn test_path_util_find_up() {
        let tmp = std::env::temp_dir();
        let found = PathUtil::find_up(&tmp, "Cargo.toml");
        assert!(found.is_some());
    }

    #[test]
    fn test_normalize() {
        let p = Path::new("/a/b/../c/./d");
        let n = PathUtil::normalize(p);
        assert_eq!(n, Path::new("a/c/d"));
    }

    #[test]
    fn test_relative_components() {
        let from = Path::new("/a/b/c");
        let to = Path::new("/a/b/d/e");
        let rel = PathUtil::relative_components(from, to);
        assert_eq!(rel, Path::new("../d/e"));
    }
}
