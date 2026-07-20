pub mod glob;
pub mod temp;
pub mod r#virtual;
pub mod watch;

use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use memmap2::Mmap;
use tokio::sync::Semaphore;

#[derive(thiserror::Error, Debug)]
pub enum FsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path not found: {0}")]
    NotFound(String),
    #[error("Path already exists: {0}")]
    AlreadyExists(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    Other(String),
}

impl From<String> for FsError {
    fn from(s: String) -> Self {
        FsError::Other(s)
    }
}

pub use crate::glob::{Glob, GlobBuilder, GlobPattern};
pub use crate::temp::{TempDir, TempFile};
pub use crate::r#virtual::InMemoryFS;
pub use crate::watch::{FileWatcher, FileWatcherBuilder};

static IO_SEMAPHORE: std::sync::LazyLock<Semaphore> =
    std::sync::LazyLock::new(|| Semaphore::new(32));

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

pub struct FileSystem;

impl FileSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn batch_read_dir(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            entries.push(self.stat(&entry.path())?);
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    pub fn batch_stat(paths: &[PathBuf]) -> Vec<anyhow::Result<FileInfo>> {
        let fs = FileSystem::new();
        paths.iter().map(|p| fs.stat(p)).collect()
    }

    pub fn batch_read_string(paths: &[PathBuf]) -> Vec<anyhow::Result<String>> {
        paths
            .iter()
            .map(|p| std::fs::read_to_string(p).map_err(|e| anyhow::anyhow!("{e}")))
            .collect()
    }

    pub fn batch_read(paths: &[PathBuf]) -> Vec<anyhow::Result<Vec<u8>>> {
        paths
            .iter()
            .map(|p| std::fs::read(p).map_err(|e| anyhow::anyhow!("{e}")))
            .collect()
    }

    pub async fn read_async(&self, path: &Path) -> anyhow::Result<Vec<u8>> {
        let _permit = IO_SEMAPHORE.acquire().await;
        Ok(tokio::fs::read(path).await?)
    }

    pub async fn read_string_async(&self, path: &Path) -> anyhow::Result<String> {
        let _permit = IO_SEMAPHORE.acquire().await;
        let data = tokio::fs::read(path).await?;
        Ok(String::from_utf8(data)?)
    }

    pub async fn write_async(&self, path: &Path, data: &[u8]) -> anyhow::Result<()> {
        let _permit = IO_SEMAPHORE.acquire().await;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    pub async fn write_string_async(&self, path: &Path, data: &str) -> anyhow::Result<()> {
        self.write_async(path, data.as_bytes()).await
    }

    pub async fn copy_async(&self, from: &Path, to: &Path) -> anyhow::Result<u64> {
        let _permit = IO_SEMAPHORE.acquire().await;
        if let Some(parent) = to.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::copy(from, to).await.map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub async fn exists_async(&self, path: &Path) -> bool {
        tokio::fs::try_exists(path).await.unwrap_or(false)
    }

    pub async fn create_dir_async(&self, path: &Path) -> anyhow::Result<()> {
        let _permit = IO_SEMAPHORE.acquire().await;
        Ok(tokio::fs::create_dir_all(path).await?)
    }

    pub async fn remove_async(&self, path: &Path) -> anyhow::Result<()> {
        let _permit = IO_SEMAPHORE.acquire().await;
        let meta = tokio::fs::symlink_metadata(path).await?;
        if meta.is_dir() {
            tokio::fs::remove_dir_all(path).await?;
        } else {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }

    pub async fn read_dir_async(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>> {
        let _permit = IO_SEMAPHORE.acquire().await;
        let mut read_dir = tokio::fs::read_dir(path).await?;
        let mut entries = Vec::new();
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            let meta = entry.metadata().await?;
            entries.push(FileInfo {
                path,
                size: meta.len(),
                is_dir: meta.is_dir(),
                is_file: meta.is_file(),
                is_symlink: meta.is_symlink(),
                modified: meta.modified().ok(),
                created: meta.created().ok(),
                accessed: meta.accessed().ok(),
                permissions: Some(perm_string(&meta.permissions())),
                readonly: meta.permissions().readonly(),
            });
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    pub async fn stat_async(&self, path: &Path) -> anyhow::Result<FileInfo> {
        let _permit = IO_SEMAPHORE.acquire().await;
        let meta = tokio::fs::symlink_metadata(path).await?;
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

    pub fn read_sync(&self, path: &Path) -> anyhow::Result<Vec<u8>> {
        Ok(std::fs::read(path)?)
    }

    pub fn read_string_sync(&self, path: &Path) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    pub fn write_sync(&self, path: &Path, data: &[u8]) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::write(path, data)?)
    }

    pub fn write_string_sync(&self, path: &Path, data: &str) -> anyhow::Result<()> {
        self.write_sync(path, data.as_bytes())
    }

    pub fn append_sync(&self, path: &Path, data: &[u8]) -> anyhow::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(file.write_all(data)?)
    }

    pub fn append_string_sync(&self, path: &Path, data: &str) -> anyhow::Result<()> {
        self.append_sync(path, data.as_bytes())
    }

    pub fn truncate_sync(&self, path: &Path, len: u64) -> anyhow::Result<()> {
        let file = std::fs::OpenOptions::new().write(true).open(path)?;
        file.set_len(len)?;
        Ok(())
    }

    pub fn copy_sync(&self, from: &Path, to: &Path) -> anyhow::Result<u64> {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::copy(from, to)?)
    }

    pub fn copy_with_progress_sync<F: FnMut(u64, u64)>(
        &self,
        from: &Path,
        to: &Path,
        mut progress: F,
    ) -> anyhow::Result<u64> {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let total = std::fs::metadata(from)?.len();
        let mut reader = std::fs::File::open(from)?;
        let mut writer = std::fs::File::create(to)?;
        let mut buf = [0u8; 65536];
        let mut copied: u64 = 0;
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            writer.write_all(&buf[..n])?;
            copied += n as u64;
            progress(copied, total);
        }
        Ok(copied)
    }

    pub fn read_dir_sync(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            entries.push(self.stat(&entry.path())?);
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    pub fn read_dir_recursive_sync(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>> {
        use rayon::prelude::*;
        let entries: Vec<_> = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .filter_map(|e| e.ok())
            .collect();
        entries
            .par_iter()
            .map(|entry| self.stat(entry.path()))
            .collect()
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

    pub fn chmod_sync(&self, path: &Path, mode: u32) -> anyhow::Result<()> {
        #[cfg(unix)]
        {
            let perm = std::fs::Permissions::from_mode(mode & 0o777);
            Ok(std::fs::set_permissions(path, perm)?)
        }
        #[cfg(not(unix))]
        {
            let _ = mode;
            anyhow::bail!("chmod not supported on this platform")
        }
    }

    pub fn set_readonly_sync(&self, path: &Path, readonly: bool) -> anyhow::Result<()> {
        let mut perm = path.metadata()?.permissions();
        perm.set_readonly(readonly);
        Ok(std::fs::set_permissions(path, perm)?)
    }

    pub fn symlink_sync(&self, target: &Path, link: &Path) -> anyhow::Result<()> {
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

    pub fn read_link_sync(&self, path: &Path) -> anyhow::Result<PathBuf> {
        Ok(std::fs::read_link(path)?)
    }

    pub fn hard_link_sync(&self, target: &Path, link: &Path) -> anyhow::Result<()> {
        Ok(std::fs::hard_link(target, link)?)
    }

    pub fn watcher_sync(&self) -> FileWatcherBuilder {
        FileWatcherBuilder::new()
    }

    pub fn temp_dir_sync(&self) -> anyhow::Result<PathBuf> {
        Ok(std::env::temp_dir())
    }

    pub fn temp_file_sync(&self) -> anyhow::Result<PathBuf> {
        let (_, path) = tempfile::NamedTempFile::new()?.keep()?;
        Ok(path)
    }

    pub fn mmap_sync(&self, path: &Path) -> anyhow::Result<Mmap> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(mmap)
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}

pub fn read_to_string(path: &Path) -> anyhow::Result<String> {
    FileSystem::new().read_string_sync(path)
}

pub fn write_string(path: &Path, data: &str) -> anyhow::Result<()> {
    FileSystem::new().write_string_sync(path, data)
}

pub fn copy_file(from: &Path, to: &Path) -> anyhow::Result<u64> {
    FileSystem::new().copy_sync(from, to)
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
    use crate::r#virtual::VirtualFileSystem;
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
        fs.write_string_sync(&file, "Hello, World!").unwrap();
        assert_eq!(fs.read_string_sync(&file).unwrap(), "Hello, World!");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_batch_stat() {
        let dir = test_dir();
        let files: Vec<PathBuf> = (0..3)
            .map(|i| {
                let p = dir.join(format!("f{i}.txt"));
                std::fs::write(&p, format!("data{i}")).ok();
                p
            })
            .collect();
        let results = FileSystem::batch_stat(&files);
        assert_eq!(results.len(), 3);
        for r in &results {
            assert!(r.is_ok());
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_batch_read() {
        let dir = test_dir();
        let files: Vec<PathBuf> = (0..3)
            .map(|i| {
                let p = dir.join(format!("r{i}.txt"));
                std::fs::write(&p, format!("data{i}")).ok();
                p
            })
            .collect();
        let results = FileSystem::batch_read(&files);
        assert_eq!(results.len(), 3);
        for r in &results {
            assert!(r.is_ok());
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_async_read_write() {
        let dir = test_dir();
        let fs = FileSystem::new();
        let file = dir.join("async_test.txt");
        fs.write_async(&file, b"async data").await.unwrap();
        let data = fs.read_async(&file).await.unwrap();
        assert_eq!(data, b"async data");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_glob_matches() {
        let pattern = GlobPattern::new("*.rs");
        assert!(pattern.matches(Path::new("main.rs")));
        assert!(!pattern.matches(Path::new("main.ts")));
    }

    #[test]
    fn test_in_memory_fs() {
        let fs = InMemoryFS::new();
        fs.write_str(Path::new("/test.txt"), "hello").unwrap();
        assert!(fs.exists(Path::new("/test.txt")));
        assert_eq!(
            fs.read_to_string(Path::new("/test.txt")).unwrap(),
            "hello"
        );
        let entries = fs.read_dir(Path::new("/")).unwrap();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_atomic_write() {
        let dir = test_dir();
        let path = dir.join("atomic.txt");
        temp::atomic_write_str(&path, "atomic data").unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "atomic data"
        );
    }
}
