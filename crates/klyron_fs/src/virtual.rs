use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::FileInfo;

pub trait VirtualFileSystem: Send + Sync {
    fn read(&self, path: &Path) -> anyhow::Result<Vec<u8>>;
    fn write(&self, path: &Path, data: &[u8]) -> anyhow::Result<()>;
    fn read_to_string(&self, path: &Path) -> anyhow::Result<String>;
    fn write_str(&self, path: &Path, data: &str) -> anyhow::Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn remove(&self, path: &Path) -> anyhow::Result<()>;
    fn create_dir(&self, path: &Path) -> anyhow::Result<()>;
    fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>>;
    fn stat(&self, path: &Path) -> anyhow::Result<FileInfo>;
    fn copy(&self, from: &Path, to: &Path) -> anyhow::Result<u64>;
    fn rename(&self, from: &Path, to: &Path) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
struct VirtualNode {
    data: Vec<u8>,
    is_dir: bool,
    created: SystemTime,
    modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct InMemoryFS {
    nodes: Arc<Mutex<HashMap<PathBuf, VirtualNode>>>,
}

impl InMemoryFS {
    pub fn new() -> Self {
        let nodes = Arc::new(Mutex::new(HashMap::new()));
        let fs = Self { nodes };
        fs.create_dir(Path::new("/")).ok();
        fs
    }

    fn ensure_parent(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !self.exists(parent) {
                self.create_dir(parent)?;
            }
        }
        Ok(())
    }
}

impl Default for InMemoryFS {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualFileSystem for InMemoryFS {
    fn read(&self, path: &Path) -> anyhow::Result<Vec<u8>> {
        let nodes = self.nodes.lock().unwrap();
        let node = nodes
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", path.display()))?;
        if node.is_dir {
            anyhow::bail!("Is a directory: {}", path.display());
        }
        Ok(node.data.clone())
    }

    fn write(&self, path: &Path, data: &[u8]) -> anyhow::Result<()> {
        self.ensure_parent(path)?;
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(
            path.to_path_buf(),
            VirtualNode {
                data: data.to_vec(),
                is_dir: false,
                created: SystemTime::now(),
                modified: SystemTime::now(),
            },
        );
        Ok(())
    }

    fn read_to_string(&self, path: &Path) -> anyhow::Result<String> {
        let data = self.read(path)?;
        Ok(String::from_utf8(data)?)
    }

    fn write_str(&self, path: &Path, data: &str) -> anyhow::Result<()> {
        self.write(path, data.as_bytes())
    }

    fn exists(&self, path: &Path) -> bool {
        self.nodes.lock().unwrap().contains_key(path)
    }

    fn remove(&self, path: &Path) -> anyhow::Result<()> {
        let mut nodes = self.nodes.lock().unwrap();
        if nodes.remove(path).is_some() {
            nodes.retain(|p, _| !p.starts_with(path));
            Ok(())
        } else {
            anyhow::bail!("Path not found: {}", path.display());
        }
    }

    fn create_dir(&self, path: &Path) -> anyhow::Result<()> {
        let mut nodes = self.nodes.lock().unwrap();
        if !nodes.contains_key(path) {
            nodes.insert(
                path.to_path_buf(),
                VirtualNode {
                    data: Vec::new(),
                    is_dir: true,
                    created: SystemTime::now(),
                    modified: SystemTime::now(),
                },
            );
        }
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>> {
        let nodes = self.nodes.lock().unwrap();
        let mut entries = Vec::new();
        for (p, node) in nodes.iter() {
            if let Some(parent) = p.parent() {
                if parent == path {
                    entries.push(FileInfo {
                        path: p.clone(),
                        size: node.data.len() as u64,
                        is_dir: node.is_dir,
                        is_file: !node.is_dir,
                        is_symlink: false,
                        modified: Some(node.modified),
                        created: Some(node.created),
                        accessed: Some(node.modified),
                        permissions: Some("rw-r--r--".into()),
                        readonly: false,
                    });
                }
            }
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    fn stat(&self, path: &Path) -> anyhow::Result<FileInfo> {
        let nodes = self.nodes.lock().unwrap();
        let node = nodes
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("Path not found: {}", path.display()))?;
        Ok(FileInfo {
            path: path.to_path_buf(),
            size: node.data.len() as u64,
            is_dir: node.is_dir,
            is_file: !node.is_dir,
            is_symlink: false,
            modified: Some(node.modified),
            created: Some(node.created),
            accessed: Some(node.modified),
            permissions: Some("rw-r--r--".into()),
            readonly: false,
        })
    }

    fn copy(&self, from: &Path, to: &Path) -> anyhow::Result<u64> {
        let data = self.read(from)?;
        let len = data.len() as u64;
        self.write(to, &data)?;
        Ok(len)
    }

    fn rename(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        let data = self.read(from)?;
        self.write(to, &data)?;
        self.remove(from)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct OverlayFS {
    upper: Arc<dyn VirtualFileSystem>,
    lower: Arc<dyn VirtualFileSystem>,
}

impl OverlayFS {
    pub fn new(upper: Arc<dyn VirtualFileSystem>, lower: Arc<dyn VirtualFileSystem>) -> Self {
        Self { upper, lower }
    }
}

impl VirtualFileSystem for OverlayFS {
    fn read(&self, path: &Path) -> anyhow::Result<Vec<u8>> {
        self.upper
            .read(path)
            .or_else(|_| self.lower.read(path))
    }

    fn write(&self, path: &Path, data: &[u8]) -> anyhow::Result<()> {
        self.upper.write(path, data)
    }

    fn read_to_string(&self, path: &Path) -> anyhow::Result<String> {
        self.upper
            .read_to_string(path)
            .or_else(|_| self.lower.read_to_string(path))
    }

    fn write_str(&self, path: &Path, data: &str) -> anyhow::Result<()> {
        self.upper.write_str(path, data)
    }

    fn exists(&self, path: &Path) -> bool {
        self.upper.exists(path) || self.lower.exists(path)
    }

    fn remove(&self, path: &Path) -> anyhow::Result<()> {
        self.upper.remove(path)
    }

    fn create_dir(&self, path: &Path) -> anyhow::Result<()> {
        self.upper.create_dir(path)
    }

    fn read_dir(&self, path: &Path) -> anyhow::Result<Vec<FileInfo>> {
        let mut entries = self.lower.read_dir(path).unwrap_or_default();
        if let Ok(upper_entries) = self.upper.read_dir(path) {
            for e in upper_entries {
                if !entries.iter().any(|x| x.path == e.path) {
                    entries.push(e);
                }
            }
        }
        Ok(entries)
    }

    fn stat(&self, path: &Path) -> anyhow::Result<FileInfo> {
        self.upper.stat(path).or_else(|_| self.lower.stat(path))
    }

    fn copy(&self, from: &Path, to: &Path) -> anyhow::Result<u64> {
        let data = self.read(from)?;
        let len = data.len() as u64;
        self.write(to, &data)?;
        Ok(len)
    }

    fn rename(&self, from: &Path, to: &Path) -> anyhow::Result<()> {
        let data = self.read(from)?;
        self.write(to, &data)?;
        self.remove(from)?;
        Ok(())
    }
}
