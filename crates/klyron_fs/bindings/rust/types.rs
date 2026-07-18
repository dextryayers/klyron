use std::path::{Path, PathBuf};
use std::time::SystemTime;

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
}
impl FileWatcherBuilder {
    pub fn new() -> Self { Self { paths: Vec::new(), recursive: true } }
    pub fn watch(mut self, path: &Path) -> Self { self.paths.push(path.to_path_buf()); self }
    pub fn recursive(mut self, recursive: bool) -> Self { self.recursive = recursive; self }
    pub fn build(self) -> anyhow::Result<FileWatcher> { Ok(FileWatcher { paths: self.paths, recursive: self.recursive }) }
}
impl Default for FileWatcherBuilder { fn default() -> Self { Self::new() } }

pub struct FileWatcher {
    pub paths: Vec<PathBuf>,
    pub recursive: bool,
}
impl FileWatcher {
    pub fn receiver(&self) -> std::sync::mpsc::Receiver<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        let paths = self.paths.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if tx.send(()).is_err() { break; }
            }
        });
        rx
    }
    pub fn start(&self) -> anyhow::Result<()> { Ok(()) }
    pub fn stop(&self) {}
}

impl FileSystem {
    pub fn new() -> Self { Self }
    pub fn read(&self, path: &Path) -> anyhow::Result<Vec<u8>> { Ok(std::fs::read(path)?) }
    pub fn read_string(&self, path: &Path) -> anyhow::Result<String> { Ok(std::fs::read_to_string(path)?) }
    pub fn write(&self, path: &Path, data: &[u8]) -> anyhow::Result<()> { if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; } Ok(std::fs::write(path, data)?) }
    pub fn write_string(&self, path: &Path, data: &str) -> anyhow::Result<()> { self.write(path, data.as_bytes()) }
    pub fn append(&self, path: &Path, data: &[u8]) -> anyhow::Result<()> { use std::io::Write; let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?; Ok(f.write_all(data)?) }
    pub fn append_string(&self, path: &Path, data: &str) -> anyhow::Result<()> { self.append(path, data.as_bytes()) }
    pub fn truncate(&self, path: &Path, len: u64) -> anyhow::Result<()> { let f = std::fs::OpenOptions::new().write(true).open(path)?; f.set_len(len)?; Ok(()) }
    pub fn copy(&self, from: &Path, to: &Path) -> anyhow::Result<u64> { if let Some(p) = to.parent() { std::fs::create_dir_all(p)?; } Ok(std::fs::copy(from, to)?) }
    pub fn move_file(&self, from: &Path, to: &Path) -> anyhow::Result<()> { if let Some(p) = to.parent() { std::fs::create_dir_all(p)?; } Ok(std::fs::rename(from, to)?) }
    pub fn remove(&self, path: &Path) -> anyhow::Result<()> { if path.is_symlink() || path.is_file() { std::fs::remove_file(path)?; } else { std::fs::remove_dir_all(path)?; } Ok(()) }
    pub fn exists(&self, path: &Path) -> bool { path.exists() }
    pub fn create_dir(&self, path: &Path) -> anyhow::Result<()> { Ok(std::fs::create_dir_all(path)?) }
    pub fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>> { let mut v = Vec::new(); for e in std::fs::read_dir(path)? { let e = e?; v.push(self.stat(&e.path())?); } v.sort_by(|a,b| a.path.cmp(&b.path)); Ok(v) }
    pub fn stat(&self, path: &Path) -> anyhow::Result<FileInfo> { let m = path.symlink_metadata()?; Ok(FileInfo { path: path.to_path_buf(), size: m.len(), is_dir: m.is_dir(), is_file: m.is_file(), is_symlink: m.is_symlink(), modified: m.modified().ok(), created: m.created().ok(), accessed: m.accessed().ok(), permissions: None, readonly: m.permissions().readonly() }) }
    pub fn chmod(&self, _path: &Path, _mode: u32) -> anyhow::Result<()> { Ok(()) }
    pub fn symlink(&self, target: &Path, link: &Path) -> anyhow::Result<()> { Ok(std::os::unix::fs::symlink(target, link)?) }
    pub fn read_link(&self, path: &Path) -> anyhow::Result<PathBuf> { Ok(std::fs::read_link(path)?) }
    pub fn watcher(&self) -> FileWatcherBuilder { FileWatcherBuilder::new() }
}
impl Default for FileSystem { fn default() -> Self { Self::new() } }
