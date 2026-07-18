use std::path::Path;
use std::fs;
use std::io::Read;

pub struct JSCFileSystem;

impl JSCFileSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        let mut file = fs::File::open(path).map_err(|e| format!("fs.readFile: {e}"))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|e| format!("fs.readFile: {e}"))?;
        Ok(buf)
    }

    pub fn write_file(&self, path: &str, data: &[u8]) -> Result<(), String> {
        fs::write(path, data).map_err(|e| format!("fs.writeFile: {e}"))
    }

    pub fn read_to_string(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| format!("fs.readFile: {e}"))
    }

    pub fn stat(&self, path: &str) -> Result<fs::Metadata, String> {
        fs::metadata(path).map_err(|e| format!("fs.stat: {e}"))
    }

    pub fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    pub fn mkdir(&self, path: &str, _mode: u32) -> Result<(), String> {
        fs::create_dir(path).map_err(|e| format!("fs.mkdir: {e}"))
    }

    pub fn mkdir_all(&self, path: &str, _mode: u32) -> Result<(), String> {
        fs::create_dir_all(path).map_err(|e| format!("fs.mkdirAll: {e}"))
    }

    pub fn read_dir(&self, path: &str) -> Result<Vec<String>, String> {
        let entries = fs::read_dir(path).map_err(|e| format!("fs.readdir: {e}"))?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("fs.readdir: {e}"))?;
            names.push(entry.file_name().to_string_lossy().to_string());
        }
        names.sort();
        Ok(names)
    }

    pub fn remove_file(&self, path: &str) -> Result<(), String> {
        fs::remove_file(path).map_err(|e| format!("fs.unlink: {e}"))
    }

    pub fn remove_dir(&self, path: &str) -> Result<(), String> {
        fs::remove_dir(path).map_err(|e| format!("fs.rmdir: {e}"))
    }

    pub fn remove_dir_all(&self, path: &str) -> Result<(), String> {
        fs::remove_dir_all(path).map_err(|e| format!("fs.rm: {e}"))
    }

    pub fn rename(&self, from: &str, to: &str) -> Result<(), String> {
        fs::rename(from, to).map_err(|e| format!("fs.rename: {e}"))
    }

    pub fn copy(&self, from: &str, to: &str) -> Result<u64, String> {
        fs::copy(from, to).map_err(|e| format!("fs.copy: {e}"))
    }

    pub fn canonicalize(&self, path: &str) -> Result<String, String> {
        fs::canonicalize(path)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| format!("fs.realpath: {e}"))
    }
}

impl Default for JSCFileSystem {
    fn default() -> Self {
        Self::new()
    }
}
