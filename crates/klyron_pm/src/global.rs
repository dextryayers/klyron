use crate::PmError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPackageInfo {
    pub name: String,
    pub version: String,
    pub installed_at: String,
    pub bin: Option<Vec<String>>,
}

fn klyron_home() -> PathBuf {
    if let Ok(val) = std::env::var("KLYRON_HOME") {
        PathBuf::from(val)
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".klyron")
    }
}

fn global_dir() -> PathBuf {
    klyron_home().join("global")
}

fn bin_dir() -> PathBuf {
    klyron_home().join("bin")
}

fn manifest_path() -> PathBuf {
    global_dir().join("manifest.json")
}

fn load_manifest() -> HashMap<String, GlobalPackageInfo> {
    let path = manifest_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn save_manifest(manifest: &HashMap<String, GlobalPackageInfo>) -> Result<(), PmError> {
    let path = manifest_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(manifest)?)?;
    Ok(())
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let days = secs / 86400;
    let t = secs % 86400;
    let h = t / 3600;
    let m = (t % 3600) / 60;
    let s = t % 60;
    let mut y = 1970i64;
    let mut rem = days as i64;
    loop {
        let di = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
        if rem < di { break; }
        rem -= di;
        y += 1;
    }
    let md = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mo = 1;
    for &d in &md {
        if rem < d { break; }
        rem -= d;
        mo += 1;
    }
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, rem + 1, h, m, s)
}

pub fn install_global(package_name: &str, version: &str) -> Result<GlobalPackageInfo, PmError> {
    let global = global_dir();
    let pkg_dir = global.join(package_name);
    std::fs::create_dir_all(&pkg_dir)?;

    let pkg_json_path = pkg_dir.join("package.json");
    let pkg_json = serde_json::json!({
        "name": package_name,
        "version": version,
        "private": true,
    });
    std::fs::write(&pkg_json_path, serde_json::to_string_pretty(&pkg_json)?)?;

    let bin_paths = Vec::new();

    let info = GlobalPackageInfo {
        name: package_name.to_string(),
        version: version.to_string(),
        installed_at: now_iso(),
        bin: if bin_paths.is_empty() { None } else { Some(bin_paths) },
    };

    let mut manifest = load_manifest();
    manifest.insert(package_name.to_string(), info.clone());
    save_manifest(&manifest)?;

    let bin = bin_dir();
    std::fs::create_dir_all(&bin)?;

    let bin_source = pkg_dir;
    if bin_source.is_dir() {
        let link_path = bin.join(package_name);
        if link_path.exists() || link_path.is_symlink() {
            let _ = std::fs::remove_file(&link_path);
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&bin_source, &link_path)
            .map_err(|e| PmError::IoError(format!("Failed to create bin symlink: {e}")))?;
    }

    Ok(info)
}

pub fn remove_global(package_name: &str) -> Result<(), PmError> {
    let mut manifest = load_manifest();
    if manifest.remove(package_name).is_none() {
        return Err(PmError::PackageNotFound(format!("Global package '{package_name}' not found")));
    }
    save_manifest(&manifest)?;

    let pkg_dir = global_dir().join(package_name);
    if pkg_dir.exists() {
        std::fs::remove_dir_all(&pkg_dir)?;
    }

    let link_path = bin_dir().join(package_name);
    if link_path.exists() || link_path.is_symlink() {
        if link_path.is_symlink() || link_path.is_file() {
            let _ = std::fs::remove_file(&link_path);
        } else {
            std::fs::remove_dir_all(&link_path)?;
        }
    }

    Ok(())
}

pub fn list_global() -> Vec<GlobalPackageInfo> {
    let manifest = load_manifest();
    let mut pkgs: Vec<_> = manifest.into_values().collect();
    pkgs.sort_by(|a, b| a.name.cmp(&b.name));
    pkgs
}

pub fn get_global_path(package_name: &str) -> Option<PathBuf> {
    let pkg_dir = global_dir().join(package_name);
    if pkg_dir.exists() {
        Some(pkg_dir)
    } else {
        None
    }
}
