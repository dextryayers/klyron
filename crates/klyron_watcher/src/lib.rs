use crossbeam_channel::unbounded;
use glob::Pattern;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum WatcherError {
  #[error("Notify error: {0}")]
  NotifyError(String),
  #[error("Watch path not found: {0}")]
  PathNotFound(String),
  #[error("Channel error: {0}")]
  ChannelError(String),
  #[error("Backend unavailable: {0}")]
  BackendUnavailable(String),
  #[error("Pattern error: {0}")]
  PatternError(String),
  #[error("Watch error: {0}")]
  WatchError(String),
}

impl From<notify::Error> for WatcherError {
  fn from(e: notify::Error) -> Self {
    Self::NotifyError(e.to_string())
  }
}

impl<T> From<crossbeam_channel::SendError<T>> for WatcherError {
  fn from(e: crossbeam_channel::SendError<T>) -> Self {
    Self::ChannelError(e.to_string())
  }
}

// ── Watch Events ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
  Create(PathBuf),
  Modify(PathBuf),
  Remove(PathBuf),
  Rename(PathBuf, PathBuf),
  Any(PathBuf),
}

// ── HMR Update ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HmrUpdate {
  pub added: Vec<PathBuf>,
  pub changed: Vec<PathBuf>,
  pub removed: Vec<PathBuf>,
  pub timestamp: SystemTime,
}

// ── Watcher Builder ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct WatcherBuilder {
  paths: Vec<PathBuf>,
  recursive: bool,
  debounce_ms: u64,
  ignore_patterns: Vec<String>,
  follow_symlinks: bool,
  poll_interval_ms: u64,
}

#[allow(dead_code)]
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
        ".DS_Store".into(),
        "*.swp".into(),
        "*.swo".into(),
        ".vscode/*".into(),
        ".idea/*".into(),
      ],
      follow_symlinks: false,
      poll_interval_ms: 200,
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

  pub fn follow_symlinks(mut self, v: bool) -> Self {
    self.follow_symlinks = v;
    self
  }

  pub fn poll_interval(mut self, ms: u64) -> Self {
    self.poll_interval_ms = ms;
    self
  }

  pub fn build(self) -> Result<FileWatcher, WatcherError> {
    let compiled: Vec<Pattern> = self
      .ignore_patterns
      .iter()
      .filter_map(|p| Pattern::new(p).ok())
      .collect();

    for path in &self.paths {
      if !path.exists() {
        return Err(WatcherError::PathNotFound(path.display().to_string()));
      }
    }

    Ok(FileWatcher {
      paths: self.paths,
      recursive: self.recursive,
      debounce: Duration::from_millis(self.debounce_ms),
      ignore_patterns: compiled,
      follow_symlinks: self.follow_symlinks,
      poll_interval: Duration::from_millis(self.poll_interval_ms),
      running: Arc::new(AtomicBool::new(false)),
      watch_handle: Arc::new(Mutex::new(None)),
    })
  }
}

impl Default for WatcherBuilder {
  fn default() -> Self {
    Self::new()
  }
}

// ── File Watcher ─────────────────────────────────────────────────────────────

pub struct FileWatcher {
  paths: Vec<PathBuf>,
  recursive: bool,
  debounce: Duration,
  ignore_patterns: Vec<Pattern>,
  #[allow(dead_code)]
  follow_symlinks: bool,
  poll_interval: Duration,
  running: Arc<AtomicBool>,
  watch_handle: Arc<Mutex<Option<RecommendedWatcher>>>,
}

impl FileWatcher {
  // ── Native OS Backend ────────────────────────────────────────────────────

  pub fn start<F>(self, callback: F) -> Result<(), WatcherError>
  where
    F: Fn(WatchEvent) + Send + 'static,
  {
    let (tx, rx) = unbounded::<WatchEvent>();
    let running = self.running.clone();
    running.store(true, Ordering::SeqCst);

    // Create native watcher
    let native_tx = tx.clone();
    let mut watcher = RecommendedWatcher::new(
      move |event: Result<Event, notify::Error>| {
        if let Ok(event) = event {
          for path in event.paths {
            let we = match event.kind {
              EventKind::Create(_) => WatchEvent::Create(path),
              EventKind::Modify(_) => WatchEvent::Modify(path),
              EventKind::Remove(_) => WatchEvent::Remove(path),
              _ => WatchEvent::Any(path),
            };
            let _ = native_tx.send(we);
          }
        }
      },
      Config::default(),
    )
    .map_err(|e| WatcherError::NotifyError(e.to_string()))?;

    // Watch paths
    for path in &self.paths {
      let mode = if self.recursive {
        RecursiveMode::Recursive
      } else {
        RecursiveMode::NonRecursive
      };
      watcher
        .watch(path, mode)
        .map_err(|e| WatcherError::WatchError(format!("Failed to watch {}: {e}", path.display())))?;
    }

    // Store watcher handle
    if let Ok(mut handle) = self.watch_handle.lock() {
      *handle = Some(watcher);
    }

    // Event dispatcher thread
    let running_clone = self.running.clone();
    let ignore = self.ignore_patterns.clone();

    thread::spawn(move || {
      while running_clone.load(Ordering::SeqCst) {
        match rx.try_recv() {
          Ok(event) => {
            if !Self::is_ignored_by_patterns(event_path(&event), &ignore) {
              callback(event);
            }
          }
          Err(crossbeam_channel::TryRecvError::Empty) => {
            thread::sleep(Duration::from_millis(10));
          }
          Err(crossbeam_channel::TryRecvError::Disconnected) => break,
        }
      }
    });

    Ok(())
  }

  // ── Polling Fallback ─────────────────────────────────────────────────────

  pub fn start_polling<F>(self, callback: F) -> Result<(), WatcherError>
  where
    F: Fn(WatchEvent) + Send + 'static,
  {
    let (tx, rx) = unbounded::<WatchEvent>();
    let running = self.running.clone();
    running.store(true, Ordering::SeqCst);

    let paths = self.paths.clone();
    let recursive = self.recursive;
    let ignore = self.ignore_patterns.clone();
    let poll_interval = self.poll_interval;
    let scan_tx = tx.clone();

    // Scanner thread
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

    // Event dispatcher
    let running_clone = self.running.clone();
    thread::spawn(move || {
      while running_clone.load(Ordering::SeqCst) {
        match rx.try_recv() {
          Ok(event) => callback(event),
          Err(crossbeam_channel::TryRecvError::Empty) => {
            thread::sleep(Duration::from_millis(10));
          }
          Err(crossbeam_channel::TryRecvError::Disconnected) => break,
        }
      }
    });

    Ok(())
  }

  // ── HMR ──────────────────────────────────────────────────────────────────

  pub fn start_hmr<F>(self, callback: F) -> Result<(), WatcherError>
  where
    F: Fn(HmrUpdate) + Send + 'static,
  {
    let (tx, rx) = unbounded::<WatchEvent>();
    let running = self.running.clone();
    running.store(true, Ordering::SeqCst);

    // Native watcher
    let native_tx = tx.clone();
    let mut watcher = RecommendedWatcher::new(
      move |event: Result<Event, notify::Error>| {
        if let Ok(event) = event {
          for path in event.paths {
            let we = match event.kind {
              EventKind::Create(_) => WatchEvent::Create(path),
              EventKind::Modify(_) => WatchEvent::Modify(path),
              EventKind::Remove(_) => WatchEvent::Remove(path),
              _ => WatchEvent::Any(path),
            };
            let _ = native_tx.send(we);
          }
        }
      },
      Config::default(),
    )
    .map_err(|e| WatcherError::NotifyError(e.to_string()))?;

    for path in &self.paths {
      let mode = if self.recursive {
        RecursiveMode::Recursive
      } else {
        RecursiveMode::NonRecursive
      };
      watcher
        .watch(path, mode)
        .map_err(|e| WatcherError::WatchError(format!("Failed to watch {}: {e}", path.display())))?;
    }

    if let Ok(mut handle) = self.watch_handle.lock() {
      *handle = Some(watcher);
    }

    // HMR debounce aggregator
    let running_clone = self.running.clone();
    let ignore = self.ignore_patterns.clone();
    let debounce = self.debounce;

    thread::spawn(move || {
      while running_clone.load(Ordering::SeqCst) {
        let mut added = Vec::new();
        let mut changed = Vec::new();
        let mut removed = Vec::new();
        let deadline = Instant::now() + debounce;

        loop {
          let remaining = deadline.saturating_duration_since(Instant::now());
          if remaining.is_zero() {
            break;
          }
          match rx.recv_timeout(remaining) {
            Ok(WatchEvent::Create(p)) => {
              if !Self::is_ignored_by_patterns(&p, &ignore) {
                added.push(p);
              }
            }
            Ok(WatchEvent::Modify(p)) => {
              if !Self::is_ignored_by_patterns(&p, &ignore) {
                changed.push(p);
              }
            }
            Ok(WatchEvent::Remove(p)) => {
              if !Self::is_ignored_by_patterns(&p, &ignore) {
                removed.push(p);
              }
            }
            Ok(WatchEvent::Rename(from, to)) => {
              if !Self::is_ignored_by_patterns(&from, &ignore) {
                removed.push(from);
              }
              if !Self::is_ignored_by_patterns(&to, &ignore) {
                added.push(to);
              }
            }
            Ok(WatchEvent::Any(p)) => {
              if !Self::is_ignored_by_patterns(&p, &ignore) {
                changed.push(p);
              }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => break,
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
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

    Ok(())
  }

  // ── Control ──────────────────────────────────────────────────────────────

  pub fn stop(&self) {
    self.running.store(false, Ordering::SeqCst);
  }

  pub fn is_running(&self) -> bool {
    self.running.load(Ordering::SeqCst)
  }

  // ── File Scanning (for polling) ──────────────────────────────────────────

  fn scan_files(paths: &[PathBuf], recursive: bool, ignore: &[Pattern]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
      if !path.exists() {
        continue;
      }
      if path.is_file() {
        if !Self::is_ignored_by_patterns(path, ignore) {
          files.push(path.clone());
        }
      } else if path.is_dir() {
        Self::scan_dir(path, recursive, ignore, &mut files);
      }
    }
    files
  }

  fn scan_dir(dir: &Path, recursive: bool, ignore: &[Pattern], files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
      for entry in entries.flatten() {
        let path = entry.path();
        if Self::is_ignored_by_patterns(&path, ignore) {
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

  // ── Pattern Filtering ───────────────────────────────────────────────────

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
}

// ── Helper ───────────────────────────────────────────────────────────────────

fn event_path(event: &WatchEvent) -> &Path {
  match event {
    WatchEvent::Create(p)
    | WatchEvent::Modify(p)
    | WatchEvent::Remove(p)
    | WatchEvent::Any(p) => p,
    WatchEvent::Rename(from, _) => from,
  }
}

// ── Tests ────────────────────────────────────────────────────────────────────

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
  fn test_watcher_builder_default() {
    let builder = WatcherBuilder::new();
    assert!(builder.recursive);
    assert_eq!(builder.debounce_ms, 300);
    assert!(builder.paths.is_empty());
    assert_eq!(builder.ignore_patterns.len(), 9);
  }

  #[test]
  fn test_watcher_builder_fluent() {
    let builder = WatcherBuilder::new()
      .add_path("/tmp")
      .recursive(false)
      .debounce(500)
      .ignore("*.log")
      .follow_symlinks(true);
    assert_eq!(builder.debounce_ms, 500);
    assert!(!builder.recursive);
    assert_eq!(builder.paths.len(), 1);
    assert!(builder.follow_symlinks);
    assert_eq!(builder.ignore_patterns.len(), 10);
  }

  #[test]
  fn test_build_fails_missing_path() {
    let result = WatcherBuilder::new()
      .add_path("/nonexistent_path_xyz_12345")
      .build();
    assert!(result.is_err());
  }

  #[test]
  fn test_build_success() {
    let dir = temp_dir("build_success");
    let watcher = WatcherBuilder::new()
      .add_path(dir.to_str().unwrap())
      .build();
    assert!(watcher.is_ok());
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_scan_files_single() {
    let dir = temp_dir("scan");
    let file = dir.join("test.txt");
    fs::write(&file, "hello").unwrap();
    let files = FileWatcher::scan_files(&[dir.clone()], true, &[]);
    assert!(files.contains(&file));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_scan_files_ignore() {
    let dir = temp_dir("scan_ignore");
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
    let dir = temp_dir("scan_recursive");
    let sub = dir.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("nested.txt"), "").unwrap();
    let files = FileWatcher::scan_files(&[dir.clone()], true, &[]);
    assert!(files.iter().any(|p| p.ends_with("nested.txt")));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_scan_files_non_recursive() {
    let dir = temp_dir("scan_nonrecursive");
    let sub = dir.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("nested.txt"), "").unwrap();
    let files = FileWatcher::scan_files(&[dir.clone()], false, &[]);
    assert!(!files.iter().any(|p| p.ends_with("nested.txt")));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_is_ignored_by_patterns() {
    let patterns = vec![Pattern::new("*.log").unwrap(), Pattern::new("node_modules/*").unwrap()];
    assert!(FileWatcher::is_ignored_by_patterns(Path::new("test.log"), &patterns));
    assert!(FileWatcher::is_ignored_by_patterns(Path::new("node_modules/pkg/index.js"), &patterns));
    assert!(!FileWatcher::is_ignored_by_patterns(Path::new("src/index.js"), &patterns));
  }

  #[test]
  fn test_matches_glob() {
    assert!(FileWatcher::matches_glob(Path::new("test.js"), "*.js").unwrap());
    assert!(!FileWatcher::matches_glob(Path::new("test.ts"), "*.js").unwrap());
    assert!(FileWatcher::matches_glob(Path::new("src/index.js"), "src/*.js").unwrap());
  }

  #[test]
  fn test_matches_glob_invalid_pattern() {
    let result = FileWatcher::matches_glob(Path::new("test"), "[invalid");
    assert!(result.is_err());
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
    let e2 = WatchEvent::Modify(PathBuf::from("f.js"));
    let e3 = WatchEvent::Remove(PathBuf::from("f.js"));
    let e4 = WatchEvent::Rename(PathBuf::from("a.js"), PathBuf::from("b.js"));
    let e5 = WatchEvent::Any(PathBuf::from("f.js"));
    assert!(format!("{:?}", e1).contains("f.js"));
    assert!(format!("{:?}", e4).contains("Rename"));
    assert_eq!(e1, WatchEvent::Create(PathBuf::from("f.js")));
    assert_ne!(e1, e2);
    assert_eq!(e5, WatchEvent::Any(PathBuf::from("f.js")));
  }

  #[test]
  fn test_watcher_builder_poll_interval() {
    let builder = WatcherBuilder::new().poll_interval(500);
    assert_eq!(builder.poll_interval_ms, 500);
  }

  #[test]
  fn test_event_path() {
    let e = WatchEvent::Create(PathBuf::from("test.js"));
    assert_eq!(event_path(&e), Path::new("test.js"));
    let e = WatchEvent::Rename(PathBuf::from("a.js"), PathBuf::from("b.js"));
    assert_eq!(event_path(&e), Path::new("a.js"));
  }

  #[test]
  fn test_watcher_error_types() {
    let e1 = WatcherError::PathNotFound("/tmp".into());
    let e2 = WatcherError::BackendUnavailable("inotify".into());
    let e3 = WatcherError::PatternError("invalid glob".into());
    assert!(e1.to_string().contains("/tmp"));
    assert!(e2.to_string().contains("inotify"));
    assert!(e3.to_string().contains("invalid glob"));
  }

  #[test]
  fn test_is_running() {
    let dir = temp_dir("is_running");
    let watcher = WatcherBuilder::new()
      .add_path(dir.to_str().unwrap())
      .build()
      .unwrap();
    assert!(!watcher.is_running());
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_stop() {
    let dir = temp_dir("stop");
    let watcher = WatcherBuilder::new()
      .add_path(dir.to_str().unwrap())
      .build()
      .unwrap();
    watcher.stop();
    assert!(!watcher.is_running());
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_scan_files_empty_dir() {
    let dir = temp_dir("empty");
    let files = FileWatcher::scan_files(&[dir.clone()], true, &[]);
    assert!(files.is_empty());
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_scan_files_with_subdir() {
    let dir = temp_dir("subdirs");
    fs::create_dir_all(dir.join("a/b/c")).unwrap();
    fs::write(dir.join("a/b/c/deep.txt"), "").unwrap();
    let files = FileWatcher::scan_files(&[dir.clone()], true, &[]);
    assert!(files.iter().any(|p| p.ends_with("deep.txt")));
    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_default_ignore_patterns() {
    let builder = WatcherBuilder::new();
    assert!(builder.ignore_patterns.contains(&"node_modules/*".into()));
    assert!(builder.ignore_patterns.contains(&".git/*".into()));
    assert!(builder.ignore_patterns.contains(&"target/*".into()));
  }

  #[test]
  fn test_build_with_multiple_paths() {
    let dir1 = temp_dir("multi1");
    let dir2 = temp_dir("multi2");
    let watcher = WatcherBuilder::new()
      .add_path(dir1.to_str().unwrap())
      .add_path(dir2.to_str().unwrap())
      .build();
    assert!(watcher.is_ok());
    assert_eq!(watcher.unwrap().paths.len(), 2);
    let _ = fs::remove_dir_all(&dir1);
    let _ = fs::remove_dir_all(&dir2);
  }

  #[test]
  fn test_watch_event_clone() {
    let e = WatchEvent::Create(PathBuf::from("test.js"));
    let cloned = e.clone();
    assert_eq!(e, cloned);
  }
}
