use anyhow::Result;
use glob::Pattern;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    Create(PathBuf),
    Modify(PathBuf),
    Remove(PathBuf),
    Rename(PathBuf, PathBuf),
}

#[derive(Debug, Clone)]
pub struct HmrUpdate {
    pub added: Vec<PathBuf>,
    pub changed: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    pub timestamp: SystemTime,
}

#[derive(Clone)]
pub struct WatcherBuilder {
    paths: Vec<PathBuf>,
    recursive: bool,
    debounce_ms: u64,
    ignore_patterns: Vec<String>,
}

impl WatcherBuilder {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            recursive: true,
            debounce_ms: 300,
            ignore_patterns: vec![
                "node_modules/*".into(),
                ".git/*".into(),
                "target/*".into(),
                "*.lock".into(),
            ],
        }
    }

    pub fn add_path(mut self, p: &str) -> Self {
        self.paths.push(PathBuf::from(p));
        self
    }

    pub fn recursive(mut self, v: bool) -> Self {
        self.recursive = v;
        self
    }

    pub fn debounce(mut self, ms: u64) -> Self {
        self.debounce_ms = ms;
        self
    }

    pub fn ignore(mut self, pattern: &str) -> Self {
        self.ignore_patterns.push(pattern.to_string());
        self
    }

    pub fn build(self) -> FileWatcher {
        let compiled: Vec<Pattern> = self
            .ignore_patterns
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect();
        FileWatcher {
            paths: self.paths,
            recursive: self.recursive,
            debounce: Duration::from_millis(self.debounce_ms),
            ignore_patterns: compiled,
            running: Arc::new(AtomicBool::new(false)),
            stop_tx: None,
            poll_interval: Duration::from_millis(200),
        }
    }
}

impl Default for WatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FileWatcher {
    paths: Vec<PathBuf>,
    recursive: bool,
    debounce: Duration,
    ignore_patterns: Vec<Pattern>,
    running: Arc<AtomicBool>,
    stop_tx: Option<Sender<()>>,
    poll_interval: Duration,
}

impl FileWatcher {
    pub fn start<F>(self, callback: F) -> Result<()>
    where
        F: Fn(WatchEvent) + Send + 'static,
    {
        let (event_tx, event_rx) = mpsc::channel::<WatchEvent>();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let paths = self.paths.clone();
        let recursive = self.recursive;
        let ignore = self.ignore_patterns.clone();
        let poll_interval = self.poll_interval;
        let scan_tx = event_tx.clone();

        thread::spawn(move || {
            let mut snapshots: HashMap<PathBuf, SystemTime> = HashMap::new();
            loop {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                let files = Self::scan_files(&paths, recursive, &ignore);
                let current_set: HashSet<PathBuf> = files.iter().cloned().collect();

                for path in &files {
                    if let Ok(meta) = path.metadata() {
                        if let Ok(modified) = meta.modified() {
                            match snapshots.get(path) {
                                Some(&prev) if prev != modified => {
                                    let _ = scan_tx.send(WatchEvent::Modify(path.clone()));
                                }
                                None => {
                                    let _ = scan_tx.send(WatchEvent::Create(path.clone()));
                                }
                                _ => {}
                            }
                            snapshots.insert(path.clone(), modified);
                        }
                    }
                }

                let removed: Vec<PathBuf> = snapshots
                    .keys()
                    .filter(|p| !current_set.contains(*p))
                    .cloned()
                    .collect();
                for path in removed {
                    snapshots.remove(&path);
                    let _ = scan_tx.send(WatchEvent::Remove(path));
                }

                thread::sleep(poll_interval);
            }
        });

        let running_clone = self.running.clone();
        thread::spawn(move || {
            while running_clone.load(Ordering::SeqCst) {
                match event_rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(event) => callback(event),
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        let mut watcher = self;
        watcher.stop_tx = Some(stop_tx);
        let _ = stop_rx.recv();
        Ok(())
    }

    pub fn start_hmr<F>(self, callback: F) -> Result<()>
    where
        F: Fn(HmrUpdate) + Send + 'static,
    {
        let (event_tx, event_rx) = mpsc::channel::<WatchEvent>();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        let paths = self.paths.clone();
        let recursive = self.recursive;
        let ignore = self.ignore_patterns.clone();
        let poll_interval = self.poll_interval;
        let scan_tx = event_tx.clone();

        thread::spawn(move || {
            let mut snapshots: HashMap<PathBuf, SystemTime> = HashMap::new();
            loop {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                let files = Self::scan_files(&paths, recursive, &ignore);
                let current_set: HashSet<PathBuf> = files.iter().cloned().collect();

                for path in &files {
                    if let Ok(meta) = path.metadata() {
                        if let Ok(modified) = meta.modified() {
                            match snapshots.get(path) {
                                Some(&prev) if prev != modified => {
                                    let _ = scan_tx.send(WatchEvent::Modify(path.clone()));
                                }
                                None => {
                                    let _ = scan_tx.send(WatchEvent::Create(path.clone()));
                                }
                                _ => {}
                            }
                            snapshots.insert(path.clone(), modified);
                        }
                    }
                }

                let removed: Vec<PathBuf> = snapshots
                    .keys()
                    .filter(|p| !current_set.contains(*p))
                    .cloned()
                    .collect();
                for path in removed {
                    snapshots.remove(&path);
                    let _ = scan_tx.send(WatchEvent::Remove(path));
                }

                thread::sleep(poll_interval);
            }
        });

        let running_clone = self.running.clone();
        let hmr_debounce = Duration::from_millis(std::cmp::max(
            50,
            self.debounce.as_millis() as u64 / 2,
        ));

        thread::spawn(move || {
            loop {
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }
                let mut added = Vec::new();
                let mut changed = Vec::new();
                let mut removed = Vec::new();
                let deadline = std::time::Instant::now() + hmr_debounce;

                loop {
                    let remaining = deadline
                        .saturating_duration_since(std::time::Instant::now());
                    if remaining.is_zero() {
                        break;
                    }
                    match event_rx.recv_timeout(remaining) {
                        Ok(WatchEvent::Create(p)) => added.push(p),
                        Ok(WatchEvent::Modify(p)) => changed.push(p),
                        Ok(WatchEvent::Remove(p)) => removed.push(p),
                        Ok(WatchEvent::Rename(_, _)) => {}
                        Err(mpsc::RecvTimeoutError::Timeout) => break,
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            running_clone.store(false, Ordering::SeqCst);
                            return;
                        }
                    }
                }

                if !added.is_empty() || !changed.is_empty() || !removed.is_empty() {
                    callback(HmrUpdate {
                        added,
                        changed,
                        removed,
                        timestamp: SystemTime::now(),
                    });
                }
            }
        });

        let mut watcher = self;
        watcher.stop_tx = Some(stop_tx);
        let _ = stop_rx.recv();
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn scan_files(paths: &[PathBuf], recursive: bool, ignore: &[Pattern]) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for path in paths {
            if !path.exists() {
                continue;
            }
            if path.is_file() {
                if !Self::is_ignored(path, ignore) {
                    files.push(path.clone());
                }
            } else if path.is_dir() {
                Self::scan_dir(path, recursive, ignore, &mut files);
            }
        }
        files
    }

    fn scan_dir(
        dir: &Path,
        recursive: bool,
        ignore: &[Pattern],
        files: &mut Vec<PathBuf>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if Self::is_ignored(&path, ignore) {
                    continue;
                }
                if path.is_file() {
                    files.push(path);
                } else if path.is_dir() && recursive {
                    Self::scan_dir(&path, recursive, ignore, files);
                }
            }
        }
    }

    fn is_ignored(path: &Path, ignore: &[Pattern]) -> bool {
        let path_str = path.to_string_lossy();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        ignore.iter().any(|p| p.matches(&path_str) || p.matches(&*file_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("klyron_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_watcher_builder_default() {
        let builder = WatcherBuilder::new();
        assert!(builder.recursive);
        assert_eq!(builder.debounce_ms, 300);
        assert!(builder.paths.is_empty());
        assert_eq!(builder.ignore_patterns.len(), 4);
    }

    #[test]
    fn test_watcher_builder_fluent() {
        let builder = WatcherBuilder::new()
            .add_path("/tmp")
            .recursive(false)
            .debounce(500)
            .ignore("*.log");
        assert_eq!(builder.debounce_ms, 500);
        assert!(!builder.recursive);
        assert_eq!(builder.paths.len(), 1);
        assert_eq!(builder.ignore_patterns.len(), 5);
    }

    #[test]
    fn test_build_compiles_patterns() {
        let watcher = WatcherBuilder::new()
            .add_path("/tmp")
            .ignore("*.log")
            .build();
        assert!(!watcher.ignore_patterns.is_empty());
    }

    #[test]
    fn test_scan_files_single() {
        let dir = temp_dir("watcher_scan");
        let file = dir.join("test.txt");
        fs::write(&file, "hello").unwrap();
        let files = FileWatcher::scan_files(&[dir.clone()], true, &[]);
        assert!(files.contains(&file));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_files_ignore() {
        let dir = temp_dir("watcher_ignore");
        fs::write(dir.join("keep.txt"), "").unwrap();
        fs::write(dir.join("ignore.log"), "").unwrap();
        let pattern = Pattern::new("*.log").unwrap();
        let files = FileWatcher::scan_files(&[dir.clone()], true, &[pattern]);
        assert!(files.iter().any(|p| p.ends_with("keep.txt")));
        assert!(!files.iter().any(|p| p.ends_with("ignore.log")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_files_recursive() {
        let dir = temp_dir("watcher_recursive");
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("nested.txt"), "").unwrap();
        let files = FileWatcher::scan_files(&[dir.clone()], true, &[]);
        assert!(files.iter().any(|p| p.ends_with("nested.txt")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_files_non_recursive() {
        let dir = temp_dir("watcher_nonrecursive");
        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("nested.txt"), "").unwrap();
        let files = FileWatcher::scan_files(&[dir.clone()], false, &[]);
        assert!(!files.iter().any(|p| p.ends_with("nested.txt")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_hmr_update_struct() {
        let update = HmrUpdate {
            added: vec![PathBuf::from("a.js")],
            changed: vec![],
            removed: vec![],
            timestamp: SystemTime::now(),
        };
        assert_eq!(update.added.len(), 1);
        assert!(update.changed.is_empty());
        assert!(update.removed.is_empty());
    }

    #[test]
    fn test_watch_event_debug() {
        let e1 = WatchEvent::Create(PathBuf::from("f.js"));
        let _e2 = WatchEvent::Modify(PathBuf::from("f.js"));
        let _e3 = WatchEvent::Remove(PathBuf::from("f.js"));
        let e4 = WatchEvent::Rename(PathBuf::from("a.js"), PathBuf::from("b.js"));
        assert!(format!("{:?}", e1).contains("f.js"));
        assert!(format!("{:?}", e4).contains("Rename"));
    }
}
