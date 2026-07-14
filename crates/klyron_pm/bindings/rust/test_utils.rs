#![cfg(test)]
use crate::{PackageManager, detect, install_cmd, add_cmd};

pub fn create_temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_pm_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[test]
fn test_test_utils() {
    let dir = create_temp_dir();
    let pm = detect(&dir);
    assert_eq!(pm, PackageManager::Npm);
}
