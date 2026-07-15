use crate::PmError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationInfo {
    pub package: String,
    pub version: Option<String>,
    pub message: String,
    pub deprecated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeprecationStore {
    deprecations: Vec<DeprecationInfo>,
}

impl DeprecationStore {
    fn empty() -> Self {
        Self { deprecations: Vec::new() }
    }
}

fn store_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("klyron")
        .join("deprecations.json")
}

fn load_store() -> DeprecationStore {
    let path = store_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_else(|_| DeprecationStore::empty())
    } else {
        DeprecationStore::empty()
    }
}

fn save_store(store: &DeprecationStore) -> Result<(), PmError> {
    let path = store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| PmError::IoError(e.to_string()))?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(store)?)?;
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

pub fn mark_deprecated(package: &str, version: Option<&str>, message: &str) -> Result<(), PmError> {
    let mut store = load_store();
    store.deprecations.push(DeprecationInfo {
        package: package.to_string(),
        version: version.map(String::from),
        message: message.to_string(),
        deprecated_at: now_iso(),
    });
    save_store(&store)
}

pub fn unmark_deprecated(package: &str, version: Option<&str>) -> Result<(), PmError> {
    let mut store = load_store();
    let len_before = store.deprecations.len();
    store.deprecations.retain(|d| {
        if d.package != package {
            return true;
        }
        match (&d.version, version) {
            (Some(_), Some(v)) => d.version.as_deref() != Some(v),
            (Some(_), None) => false,
            (None, Some(_)) => true,
            (None, None) => false,
        }
    });
    if store.deprecations.len() == len_before {
        return Err(PmError::IoError(format!("No deprecation found for {package}")));
    }
    save_store(&store)
}

pub fn get_deprecations(package: &str, version: Option<&str>) -> Vec<DeprecationInfo> {
    let store = load_store();
    store.deprecations.into_iter().filter(|d| {
        if d.package != package {
            return false;
        }
        match (&d.version, version) {
            (Some(_), Some(v)) => d.version.as_deref() == Some(v),
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => true,
        }
    }).collect()
}

pub fn show_deprecations(deps: &[crate::DependencyNode]) {
    let store = load_store();
    for dep in deps {
        let matches: Vec<_> = store.deprecations.iter().filter(|d| {
            if d.package != dep.name {
                return false;
            }
            match &d.version {
                Some(v) => v == &dep.version,
                None => true,
            }
        }).collect();
        for m in matches {
            eprintln!("warning: {}@{} is deprecated: {}", m.package, dep.version, m.message);
        }
    }
}
