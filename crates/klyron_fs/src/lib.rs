use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct FileSystem;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub is_file: bool,
    pub is_symlink: bool,
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub permissions: Option<String>,
    pub readonly: bool,
}

#[derive(Debug, Clone)]
pub enum FsEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Removed(PathBuf),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct FileWatcherBuilder {
    paths: Vec<PathBuf>,
    recursive: bool,
    poll_interval: Duration,
    filter: Option<Vec<String>>,
}

impl FileWatcherBuilder {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            recursive: true,
            poll_interval: Duration::from_millis(500),
            filter: None,
        }
    }

    pub fn watch(mut self, path: &Path) -> Self {
        self.paths.push(path.to_path_buf());
        self
    }

    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub fn extensions(mut self, exts: &[&str]) -> Self {
        self.filter = Some(exts.iter().map(|e| e.to_string()).collect());
        self
    }

    pub fn build(self) -> anyhow::Result<FileWatcher> {
        let (tx, rx) = mpsc::channel();
        let snapshots = Arc::new(Mutex::new(HashMap::new()));

        for path in &self.paths {
            if !path.exists() {
                anyhow::bail!("Watch path does not exist: {}", path.display());
            }
            let snapshot = collect_snapshot(path, self.recursive, self.filter.as_ref());
            snapshots.lock().unwrap().insert(path.to_path_buf(), snapshot);
        }

        Ok(FileWatcher {
            paths: self.paths.clone(),
            recursive: self.recursive,
            poll_interval: self.poll_interval,
            filter: self.filter.clone(),
            rx,
            tx,
            snapshots,
            running: Arc::new(Mutex::new(false)),
        })
    }
}

impl Default for FileWatcherBuilder {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
struct FileSnapshot {
    entries: HashMap<PathBuf, FileMeta>,
}

#[derive(Debug, Clone, PartialEq)]
struct FileMeta {
    size: u64,
    modified: Option<SystemTime>,
    exists: bool,
}

fn collect_snapshot(
    dir: &Path,
    recursive: bool,
    filter: Option<&Vec<String>>,
) -> FileSnapshot {
    let mut entries = HashMap::new();
    if dir.is_file() {
        let meta = dir.metadata().ok();
        entries.insert(
            dir.to_path_buf(),
            FileMeta {
                size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                modified: meta.and_then(|m| m.modified().ok()),
                exists: dir.exists(),
            },
        );
    } else if dir.is_dir() {
        if let Ok(read) = std::fs::read_dir(dir) {
            for entry in read.flatten() {
                let path = entry.path();
                if let Some(ref exts) = filter {
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if !exts.contains(&ext.to_string_lossy().to_string()) {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }
                }
                let meta = entry.metadata().ok();
                entries.insert(
                    path.clone(),
                    FileMeta {
                        size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                        modified: meta.and_then(|m| m.modified().ok()),
                        exists: true,
                    },
                );
                if recursive && path.is_dir() {
                    let sub = collect_snapshot(&path, true, filter);
                    entries.extend(sub.entries);
                }
            }
        }
    }
    FileSnapshot { entries }
}

fn diff_snapshots(
    old: &FileSnapshot,
    new: &FileSnapshot,
) -> Vec<FsEvent> {
    let mut events = Vec::new();
    for (path, new_meta) in &new.entries {
        match old.entries.get(path) {
            None => events.push(FsEvent::Created(path.clone())),
            Some(old_meta) => {
                if old_meta != new_meta {
                    events.push(FsEvent::Modified(path.clone()));
                }
            }
        }
    }
    for path in old.entries.keys() {
        if !new.entries.contains_key(path) {
            events.push(FsEvent::Removed(path.clone()));
        }
    }
    events
}

pub struct FileWatcher {
    paths: Vec<PathBuf>,
    recursive: bool,
    poll_interval: Duration,
    filter: Option<Vec<String>>,
    rx: mpsc::Receiver<FsEvent>,
    tx: mpsc::Sender<FsEvent>,
    snapshots: Arc<Mutex<HashMap<PathBuf, FileSnapshot>>>,
    running: Arc<Mutex<bool>>,
}

impl FileWatcher {
    pub fn receiver(&self) -> &mpsc::Receiver<FsEvent> {
        &self.rx
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let paths = self.paths.clone();
        let recursive = self.recursive;
        let interval = self.poll_interval;
        let filter = self.filter.clone();
        let tx = self.tx.clone();
        let snapshots = self.snapshots.clone();
        let running = self.running.clone();

        *running.lock().unwrap() = true;

        std::thread::spawn(move || {
            while *running.lock().unwrap() {
                std::thread::sleep(interval);
                for path in &paths {
                    let new_snapshot = collect_snapshot(path, recursive, filter.as_ref());
                    let old_snapshot = {
                        let mut map = snapshots.lock().unwrap();
                        let old = map.get(path).cloned();
                        map.insert(path.to_path_buf(), new_snapshot.clone());
                        old
                    };
                    if let Some(ref old) = old_snapshot {
                        for event in diff_snapshots(old, &new_snapshot) {
                            if tx.send(event).is_err() {
                                return;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }
}

impl FileSystem {
    pub fn new() -> Self { Self }

    pub fn read(&self, path: &Path) -> anyhow::Result<Vec<u8>> {
        Ok(std::fs::read(path)?)
    }

    pub fn read_string(&self, path: &Path) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    pub fn write(&self, path: &Path, data: &[u8]) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::write(path, data)?)
    }

    pub fn write_string(&self, path: &Path, data: &str) -> anyhow::Result<()> {
        self.write(path, data.as_bytes())
    }

    pub fn append(&self, path: &Path, data: &[u8]) -> anyhow::Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(file.write_all(data)?)
    }

    pub fn append_string(&self, path: &Path, data: &str) -> anyhow::Result<()> {
        self.append(path, data.as_bytes())
    }

    pub fn truncate(&self, path: &Path, len: u64) -> anyhow::Result<()> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(path)?;
        file.set_len(len)?;
        Ok(())
    }

    pub fn copy(&self, from: &Path, to: &Path) -> anyhow::Result<u64> {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::copy(from, to)?)
    }

    pub fn move_file(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::rename(from, to)?)
    }

    pub fn remove(&self, path: &Path) -> anyhow::Result<()> {
        if path.is_symlink() {
            std::fs::remove_file(path)?;
        } else if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    pub fn create_dir(&self, path: &Path) -> anyhow::Result<()> {
        Ok(std::fs::create_dir_all(path)?)
    }

    pub fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            entries.push(self.stat(&entry.path())?);
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    pub fn read_dir_recursive(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>> {
        let mut entries = Vec::new();
        for entry in walkdir::WalkDir::new(path).into_iter().filter_entry(|e| {
            !e.file_name().to_string_lossy().starts_with('.')
        }) {
            let entry = entry?;
            entries.push(self.stat(entry.path())?);
        }
        Ok(entries)
    }

    pub fn stat(&self, path: &Path) -> anyhow::Result<FileInfo> {
        let meta = path.symlink_metadata()?;
        Ok(FileInfo {
            path: path.to_path_buf(),
            size: meta.len(),
            is_dir: meta.is_dir(),
            is_file: meta.is_file(),
            is_symlink: meta.is_symlink(),
            modified: meta.modified().ok(),
            created: meta.created().ok(),
            accessed: meta.accessed().ok(),
            permissions: Some(perm_string(&meta.permissions())),
            readonly: meta.permissions().readonly(),
        })
    }

    pub fn chmod(&self, path: &Path, mode: u32) -> anyhow::Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perm = std::fs::Permissions::from_mode(mode & 0o777);
            Ok(std::fs::set_permissions(path, perm)?)
        }
        #[cfg(not(unix))]
        {
            let _ = mode;
            anyhow::bail!("chmod not supported on this platform")
        }
    }

    pub fn set_readonly(&self, path: &Path, readonly: bool) -> anyhow::Result<()> {
        let mut perm = path.metadata()?.permissions();
        perm.set_readonly(readonly);
        Ok(std::fs::set_permissions(path, perm)?)
    }

    pub fn symlink(&self, target: &Path, link: &Path) -> anyhow::Result<()> {
        #[cfg(unix)]
        {
            Ok(std::os::unix::fs::symlink(target, link)?)
        }
        #[cfg(windows)]
        {
            if target.is_dir() {
                Ok(std::os::windows::fs::symlink_dir(target, link)?)
            } else {
                Ok(std::os::windows::fs::symlink_file(target, link)?)
            }
        }
        #[cfg(not(any(unix, windows)))]
        {
            anyhow::bail!("symlink not supported on this platform")
        }
    }

    pub fn read_link(&self, path: &Path) -> anyhow::Result<PathBuf> {
        Ok(std::fs::read_link(path)?)
    }

    pub fn hard_link(&self, target: &Path, link: &Path) -> anyhow::Result<()> {
        Ok(std::fs::hard_link(target, link)?)
    }

    pub fn watcher(&self) -> FileWatcherBuilder {
        FileWatcherBuilder::new()
    }
}

impl Default for FileSystem {
    fn default() -> Self { Self::new() }
}

pub fn read_to_string(path: &Path) -> anyhow::Result<String> {
    FileSystem::new().read_string(path)
}

pub fn write_string(path: &Path, data: &str) -> anyhow::Result<()> {
    FileSystem::new().write_string(path, data)
}

pub fn copy_file(from: &Path, to: &Path) -> anyhow::Result<u64> {
    FileSystem::new().copy(from, to)
}

#[cfg(unix)]
fn perm_string(perm: &std::fs::Permissions) -> String {
    format!("{:o}", perm.mode() & 0o777)
}

#[cfg(not(unix))]
fn perm_string(_perm: &std::fs::Permissions) -> String {
    "rwxr-xr-x".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use std::sync::atomic::{AtomicU64, Ordering};
    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("klyron_fs_test_{id}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).ok();
        dir
    }

    #[test]
    fn test_fs_write_read() {
        let dir = test_dir();
        let fs = FileSystem::new();
        let file = dir.join("hello.txt");
        fs.write_string(&file, "Hello, World!").unwrap();
        assert_eq!(fs.read_string(&file).unwrap(), "Hello, World!");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_fs_exists() {
        let fs = FileSystem::new();
        assert!(fs.exists(Path::new("/tmp")));
        assert!(!fs.exists(Path::new("/nonexistent_path_xyz_12345")));
    }

    #[test]
    fn test_fs_append() {
        let dir = test_dir();
        let fs = FileSystem::new();
        let file = dir.join("append.txt");
        fs.write_string(&file, "line1\n").unwrap();
        fs.append_string(&file, "line2\n").unwrap();
        let content = fs.read_string(&file).unwrap();
        assert_eq!(content, "line1\nline2\n");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_fs_stat() {
        let fs = FileSystem::new();
        let info = fs.stat(Path::new("/tmp")).unwrap();
        assert!(info.is_dir);
        assert!(!info.is_file);
    }

    #[test]
    fn test_fs_create_dir() {
        let dir = test_dir();
        let fs = FileSystem::new();
        let new_dir = dir.join("nested/deep/dir");
        fs.create_dir(&new_dir).unwrap();
        assert!(fs.exists(&new_dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_fs_copy_move() {
        let dir = test_dir();
        let fs = FileSystem::new();
        let src = dir.join("src.txt");
        let dst = dir.join("dst.txt");
        let moved = dir.join("moved.txt");
        fs.write_string(&src, "data").unwrap();
        fs.copy(&src, &dst).unwrap();
        assert!(fs.exists(&dst));
        fs.move_file(&dst, &moved).unwrap();
        assert!(!fs.exists(&dst));
        assert!(fs.exists(&moved));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_fs_symlink() {
        let dir = test_dir();
        let fs = FileSystem::new();
        let target = dir.join("original.txt");
        let link = dir.join("link.txt");
        fs.write_string(&target, "linked").unwrap();
        fs.symlink(&target, &link).unwrap();
        assert!(fs.exists(&link));
        assert!(fs.stat(&link).unwrap().is_symlink || cfg!(windows));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_fs_truncate() {
        let dir = test_dir();
        let fs = FileSystem::new();
        let file = dir.join("trunc.txt");
        fs.write_string(&file, "long content here").unwrap();
        fs.truncate(&file, 4).unwrap();
        assert_eq!(fs.read_string(&file).unwrap(), "long");
        let _ = fs::remove_dir_all(&dir);
    }
}
