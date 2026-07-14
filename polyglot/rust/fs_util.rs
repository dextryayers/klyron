//! File system utilities.
//!
//! High-level helpers for reading, writing, and managing
//! files and directories.

use crate::types::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Read the entire contents of a file into a `String`.
pub fn read_file(path: impl AsRef<Path>) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

/// Write a string to a file, creating parent directories if needed.
pub fn write_file(path: impl AsRef<Path>, contents: &str) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(fs::write(path, contents)?)
}

/// Ensure a directory exists, creating it and all missing parents.
pub fn ensure_dir(path: impl AsRef<Path>) -> Result<()> {
    Ok(fs::create_dir_all(path)?)
}

/// Create a temporary directory with a unique name inside the system temp dir.
///
/// The directory is named `klyron_<counter>` where `<counter>` is an
/// auto-incrementing integer that avoids collisions.
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
