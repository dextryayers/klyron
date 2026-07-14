use std::path::Path;

pub fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 { return false; }
    parts.iter().all(|p| p.parse::<u64>().is_ok())
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.2} {}", size, UNITS[unit_idx])
}

pub fn hash_string(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn temp_dir() -> std::path::PathBuf {
    std::env::temp_dir().join("klyron")
}

pub fn ensure_temp_dir() -> anyhow::Result<std::path::PathBuf> {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn project_name_from_dir(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string())
}

pub fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_valid_semver() {
        assert!(is_valid_semver("1.2.3"));
        assert!(!is_valid_semver("1.2"));
        assert!(!is_valid_semver("abc"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "helloworld");
        assert_eq!(slugify("My-App_v2"), "my-app_v2");
    }
}
