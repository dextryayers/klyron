use std::path::{Path, PathBuf};
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
    pub modified: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
    pub permissions: Option<String>,
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
        if path.is_dir() {
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
            let meta = entry.metadata()?;
            entries.push(FileInfo {
                path: entry.path(),
                size: meta.len(),
                is_dir: meta.is_dir(),
                is_file: meta.is_file(),
                is_symlink: meta.is_symlink(),
                modified: meta.modified().ok(),
                created: meta.created().ok(),
                permissions: Some(perm_string(&meta.permissions())),
            });
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    pub fn stat(&self, path: &Path) -> anyhow::Result<FileInfo> {
        let meta = path.metadata()?;
        Ok(FileInfo {
            path: path.to_path_buf(),
            size: meta.len(),
            is_dir: meta.is_dir(),
            is_file: meta.is_file(),
            is_symlink: meta.is_symlink(),
            modified: meta.modified().ok(),
            created: meta.created().ok(),
            permissions: Some(perm_string(&meta.permissions())),
        })
    }
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
    #[test]
    fn test_fs_write_read() {
        let dir = std::env::temp_dir().join("klyron_fs_test");
        let _ = std::fs::remove_dir_all(&dir);
        let fs = FileSystem::new();
        let file = dir.join("hello.txt");
        fs.write_string(&file, "Hello, World!").unwrap();
        assert_eq!(fs.read_string(&file).unwrap(), "Hello, World!");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_fs_exists() {
        let fs = FileSystem::new();
        assert!(fs.exists(Path::new("/tmp")));
        assert!(!fs.exists(Path::new("/nonexistent_path_xyz")));
    }
}
