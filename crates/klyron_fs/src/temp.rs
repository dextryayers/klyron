use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tempfile::{Builder, NamedTempFile};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TempFile {
    _file: Option<NamedTempFile>,
    path: PathBuf,
    keep: bool,
}

impl TempFile {
    pub fn new() -> anyhow::Result<Self> {
        let file = Builder::new()
            .prefix("klyron_temp_")
            .tempfile()?;
        let path = file.path().to_path_buf();
        Ok(Self {
            _file: Some(file),
            path,
            keep: false,
        })
    }

    pub fn with_prefix(prefix: &str) -> anyhow::Result<Self> {
        let file = Builder::new()
            .prefix(prefix)
            .tempfile()?;
        let path = file.path().to_path_buf();
        Ok(Self {
            _file: Some(file),
            path,
            keep: false,
        })
    }

    pub fn with_suffix(suffix: &str) -> anyhow::Result<Self> {
        let file = Builder::new()
            .suffix(suffix)
            .tempfile()?;
        let path = file.path().to_path_buf();
        Ok(Self {
            _file: Some(file),
            path,
            keep: false,
        })
    }

    pub fn in_dir(dir: &Path) -> anyhow::Result<Self> {
        let file = Builder::new()
            .prefix("klyron_temp_")
            .tempfile_in(dir)?;
        let path = file.path().to_path_buf();
        Ok(Self {
            _file: Some(file),
            path,
            keep: false,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn keep(mut self) -> anyhow::Result<PathBuf> {
        if let Some(file) = self._file.take() {
            let (_, path) = file.keep()?;
            self.path = path.clone();
            self.keep = true;
            Ok(path)
        } else {
            Ok(self.path.clone())
        }
    }

    pub fn write(&self, data: &[u8]) -> anyhow::Result<()> {
        std::fs::write(&self.path, data)?;
        Ok(())
    }

    pub fn write_str(&self, data: &str) -> anyhow::Result<()> {
        std::fs::write(&self.path, data)?;
        Ok(())
    }

    pub fn read(&self) -> anyhow::Result<Vec<u8>> {
        Ok(std::fs::read(&self.path)?)
    }

    pub fn read_to_string(&self) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(&self.path)?)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.keep {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

pub struct TempDir {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl TempDir {
    pub fn new() -> anyhow::Result<Self> {
        let dir = tempfile::Builder::new()
            .prefix("klyron_temp_")
            .tempdir()?;
        let path = dir.path().to_path_buf();
        Ok(Self {
            _dir: dir,
            path,
        })
    }

    pub fn with_prefix(prefix: &str) -> anyhow::Result<Self> {
        let dir = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()?;
        let path = dir.path().to_path_buf();
        Ok(Self {
            _dir: dir,
            path,
        })
    }

    pub fn in_dir(parent: &Path) -> anyhow::Result<Self> {
        let dir = tempfile::Builder::new()
            .prefix("klyron_temp_")
            .tempdir_in(parent)?;
        let path = dir.path().to_path_buf();
        Ok(Self {
            _dir: dir,
            path,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn child(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    pub fn create_file(&self, name: &str, data: &[u8]) -> anyhow::Result<PathBuf> {
        let path = self.child(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, data)?;
        Ok(path)
    }

    pub fn create_dir(&self, name: &str) -> anyhow::Result<PathBuf> {
        let path = self.child(name);
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }
}

pub fn temp_file() -> anyhow::Result<TempFile> {
    TempFile::new()
}

pub fn temp_dir() -> anyhow::Result<TempDir> {
    TempDir::new()
}

pub fn temp_file_in(dir: &Path) -> anyhow::Result<TempFile> {
    TempFile::in_dir(dir)
}

pub fn temp_dir_in(dir: &Path) -> anyhow::Result<TempDir> {
    TempDir::in_dir(dir)
}

pub fn atomic_write(path: &Path, data: &[u8]) -> anyhow::Result<()> {
    let id = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp_path = path.with_extension(format!("tmp_{id}"));
    std::fs::write(&tmp_path, data)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn atomic_write_str(path: &Path, data: &str) -> anyhow::Result<()> {
    atomic_write(path, data.as_bytes())
}
