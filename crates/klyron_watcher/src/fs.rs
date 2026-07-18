use std::path::{Path, PathBuf};

use glob::Pattern;

use crate::WatchEvent;
use crate::WatcherError;

pub fn scan_files(paths: &[PathBuf], recursive: bool, ignore: &[Pattern]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if !path.exists() {
            continue;
        }
        if path.is_file() {
            if !is_ignored_by_patterns(path, ignore) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            scan_dir(path, recursive, ignore, &mut files);
        }
    }
    files
}

pub fn scan_dir(dir: &Path, recursive: bool, ignore: &[Pattern], files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if is_ignored_by_patterns(&path, ignore) {
                continue;
            }
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() && recursive {
                scan_dir(&path, recursive, ignore, files);
            }
        }
    }
}

pub fn is_ignored_by_patterns(path: &Path, patterns: &[Pattern]) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    patterns
        .iter()
        .any(|p| p.matches(&path_str) || p.matches(&*file_name))
}

pub fn matches_glob(path: &Path, pattern: &str) -> Result<bool, WatcherError> {
    let pat = Pattern::new(pattern).map_err(|e| WatcherError::PatternError(e.to_string()))?;
    Ok(pat.matches(&path.to_string_lossy()))
}

pub fn event_path(event: &WatchEvent) -> &Path {
    match event {
        WatchEvent::Create(p)
        | WatchEvent::Modify(p)
        | WatchEvent::Remove(p)
        | WatchEvent::Any(p) => p,
        WatchEvent::Rename(from, _) => from,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("klyron_test_watcher_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_scan_files_single() {
        let dir = temp_dir("scan");
        let file = dir.join("test.txt");
        fs::write(&file, "hello").unwrap();
        let files = scan_files(&[dir.clone()], true, &[]);
        assert!(files.contains(&file));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_files_ignore() {
        let dir = temp_dir("scan_ignore");
        fs::write(dir.join("keep.txt"), "").unwrap();
        fs::write(dir.join("ignore.log"), "").unwrap();
        let pattern = Pattern::new("*.log").unwrap();
        let files = scan_files(&[dir.clone()], true, &[pattern]);
        assert!(files.iter().any(|p| p.ends_with("keep.txt")));
        assert!(!files.iter().any(|p| p.ends_with("ignore.log")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_files_recursive() {
        let dir = temp_dir("scan_recursive");
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("nested.txt"), "").unwrap();
        let files = scan_files(&[dir.clone()], true, &[]);
        assert!(files.iter().any(|p| p.ends_with("nested.txt")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_files_non_recursive() {
        let dir = temp_dir("scan_nonrecursive");
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("nested.txt"), "").unwrap();
        let files = scan_files(&[dir.clone()], false, &[]);
        assert!(!files.iter().any(|p| p.ends_with("nested.txt")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_is_ignored_by_patterns() {
        let patterns = vec![
            Pattern::new("*.log").unwrap(),
            Pattern::new("node_modules/*").unwrap(),
        ];
        assert!(is_ignored_by_patterns(Path::new("test.log"), &patterns));
        assert!(is_ignored_by_patterns(
            Path::new("node_modules/pkg/index.js"),
            &patterns
        ));
        assert!(!is_ignored_by_patterns(Path::new("src/index.js"), &patterns));
    }

    #[test]
    fn test_matches_glob() {
        assert!(matches_glob(Path::new("test.js"), "*.js").unwrap());
        assert!(!matches_glob(Path::new("test.ts"), "*.js").unwrap());
        assert!(matches_glob(Path::new("src/index.js"), "src/*.js").unwrap());
    }

    #[test]
    fn test_matches_glob_invalid_pattern() {
        let result = matches_glob(Path::new("test"), "[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_event_path() {
        let e = WatchEvent::Create(PathBuf::from("test.js"));
        assert_eq!(event_path(&e), Path::new("test.js"));
        let e = WatchEvent::Rename(PathBuf::from("a.js"), PathBuf::from("b.js"));
        assert_eq!(event_path(&e), Path::new("a.js"));
    }

    #[test]
    fn test_scan_files_empty_dir() {
        let dir = temp_dir("empty");
        let files = scan_files(&[dir.clone()], true, &[]);
        assert!(files.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_files_with_subdir() {
        let dir = temp_dir("subdirs");
        fs::create_dir_all(dir.join("a/b/c")).unwrap();
        fs::write(dir.join("a/b/c/deep.txt"), "").unwrap();
        let files = scan_files(&[dir.clone()], true, &[]);
        assert!(files.iter().any(|p| p.ends_with("deep.txt")));
        let _ = fs::remove_dir_all(&dir);
    }
}
