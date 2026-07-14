use crate::types::{FileInfo, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn read_file(path: impl AsRef<Path>) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

pub fn read_file_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    Ok(fs::read(path)?)
}

pub fn write_file(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(fs::write(path, contents)?)
}

pub fn write_file_bytes(path: impl AsRef<Path>, contents: &[u8]) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(fs::write(path, contents)?)
}

pub fn append_file(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    use std::io::Write;
    let mut f = fs::OpenOptions::new().append(true).create(true).open(path)?;
    f.write_all(contents.as_bytes())?;
    Ok(())
}

pub fn ensure_dir(path: impl AsRef<Path>) -> Result<()> {
    Ok(fs::create_dir_all(path)?)
}

pub fn remove(path: impl AsRef<Path>) -> Result<()> {
    let p = path.as_ref();
    if p.is_dir() {
        fs::remove_dir_all(p)?;
    } else {
        fs::remove_file(p)?;
    }
    Ok(())
}

pub fn copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    Ok(fs::copy(src, dst).map(|_| ())?)
}

pub fn rename(old: impl AsRef<Path>, new: impl AsRef<Path>) -> Result<()> {
    Ok(fs::rename(old, new)?)
}

pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

pub fn is_dir(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_dir()
}

pub fn is_file(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}

pub fn file_size(path: impl AsRef<Path>) -> Result<u64> {
    Ok(fs::metadata(path)?.len())
}

pub fn stat(path: impl AsRef<Path>) -> Result<FileInfo> {
    let meta = fs::metadata(path.as_ref())?;
    Ok(FileInfo {
        path: path.as_ref().to_string_lossy().to_string(),
        size: meta.len(),
        is_dir: meta.is_dir(),
        is_file: meta.is_file(),
        modified: meta.modified()
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
            .unwrap_or(0),
    })
}

pub fn read_dir(path: impl AsRef<Path>) -> Result<Vec<FileInfo>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        entries.push(FileInfo {
            path: entry.path().to_string_lossy().to_string(),
            size: meta.len(),
            is_dir: meta.is_dir(),
            is_file: meta.is_file(),
            modified: meta.modified()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
                .unwrap_or(0),
        });
    }
    Ok(entries)
}

pub fn list_files(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            files.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    Ok(files)
}

pub fn temp_dir() -> Result<PathBuf> {
    let base = std::env::temp_dir();
    let mut counter = 0u64;
    loop {
        let name = format!("klyron_{}", counter);
        let path = base.join(&name);
        if !path.exists() {
            fs::create_dir(&path)?;
            return Ok(path);
        }
        counter += 1;
    }
}

pub fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn chdir(path: impl AsRef<Path>) -> Result<()> {
    Ok(std::env::set_current_dir(path)?)
}
