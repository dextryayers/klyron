use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

// ── Semver ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Semver {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Option<String>,
    pub build: Option<String>,
}

impl Semver {
    #[inline]
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self { major, minor, patch, pre: None, build: None }
    }

    #[inline]
    pub fn parse(version: &str) -> anyhow::Result<Self> {
        let version = version.trim();
        let (version, build) = match version.split_once('+') {
            Some((v, b)) => (v, Some(b.to_string())),
            None => (version, None),
        };
        let (version, pre) = match version.split_once('-') {
            Some((v, p)) => (v, Some(p.to_string())),
            None => (version, None),
        };
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            anyhow::bail!("Invalid semver: expected major.minor.patch, got {version}");
        }
        let major = parts[0].parse().map_err(|_| anyhow::anyhow!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1].parse().map_err(|_| anyhow::anyhow!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2].parse().map_err(|_| anyhow::anyhow!("Invalid patch version: {}", parts[2]))?;
        Ok(Self { major, minor, patch, pre, build })
    }

    #[inline]
    pub fn compatible(&self, other: &Semver) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl std::fmt::Display for Semver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre {
            write!(f, "-{pre}")?;
        }
        if let Some(ref build) = self.build {
            write!(f, "+{build}")?;
        }
        Ok(())
    }
}

impl FromStr for Semver {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// ── PathUtil ──────────────────────────────────────────────────────────────

pub struct PathUtil;

impl PathUtil {
    #[inline]
    pub fn join(base: &Path, segments: &[&str]) -> PathBuf {
        let mut p = base.to_path_buf();
        for s in segments {
            p = p.join(s);
        }
        p
    }

    #[inline]
    pub fn extension(p: &Path) -> &str {
        p.extension().and_then(|e| e.to_str()).unwrap_or("")
    }

    #[inline]
    pub fn file_stem(p: &Path) -> &str {
        p.file_stem().and_then(|s| s.to_str()).unwrap_or("")
    }

    #[inline]
    pub fn normalize(p: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in p.components() {
            match component {
                std::path::Component::Normal(c) => components.push(c),
                std::path::Component::ParentDir => { components.pop(); }
                _ => {}
            }
        }
        let mut result = PathBuf::new();
        for c in components {
            result = result.join(c);
        }
        result
    }

    #[inline]
    pub fn is_js_like(p: &Path) -> bool {
        matches!(Self::extension(p), "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs")
    }

    #[inline]
    pub fn to_unix(p: &Path) -> String {
        p.to_string_lossy().replace('\\', "/")
    }
}

// ── ShellUtil ─────────────────────────────────────────────────────────────

pub struct ShellUtil;

impl ShellUtil {
    #[inline]
    pub fn build_cmd(program: &str, args: &[&str]) -> std::process::Command {
        let mut cmd = std::process::Command::new(program);
        cmd.args(args);
        cmd
    }

    pub fn run_capture(program: &str, args: &[&str]) -> anyhow::Result<String> {
        let output = Self::build_cmd(program, args)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run {program}: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("{program} failed: {stderr}");
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    #[inline]
    pub fn run_interactive(program: &str, args: &[&str]) -> anyhow::Result<std::process::ExitStatus> {
        let status = Self::build_cmd(program, args)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn {program}: {e}"))?
            .wait()
            .map_err(|e| anyhow::anyhow!("Failed to wait for {program}: {e}"))?;
        Ok(status)
    }

    pub fn which(name: &str) -> Option<PathBuf> {
        let paths = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&paths) {
            let full = dir.join(name);
            if full.is_file() {
                return Some(full);
            }
            #[cfg(windows)]
            {
                let full_exe = dir.join(format!("{name}.exe"));
                if full_exe.is_file() {
                    return Some(full_exe);
                }
            }
        }
        None
    }

    #[inline]
    pub fn escape_arg(arg: &str) -> String {
        if arg.contains(' ') || arg.contains('\'') || arg.contains('"') {
            format!("\"{}\"", arg.replace('\\', "\\\\").replace('"', "\\\""))
        } else {
            arg.to_string()
        }
    }
}

// ── HashUtil ──────────────────────────────────────────────────────────────

pub struct HashUtil;

impl HashUtil {
    #[inline]
    pub fn xxhash64(data: &[u8]) -> u64 {
        use std::hash::Hasher;
        let mut hasher = twox_hash::XxHash64::default();
        hasher.write(data);
        hasher.finish()
    }

    #[inline]
    pub fn xxhash32(data: &[u8]) -> u32 {
        use std::hash::Hasher;
        let mut hasher = twox_hash::XxHash32::default();
        hasher.write(data);
        hasher.finish() as u32
    }

    #[inline]
    pub fn sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    #[inline]
    pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    #[inline]
    pub fn md5(data: &[u8]) -> String {
        let mut hasher = md5::Md5::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    #[inline]
    pub fn hash_file(path: &Path) -> anyhow::Result<String> {
        let data = std::fs::read(path)?;
        Ok(Self::sha256(&data))
    }
}

// ── UrlUtil ───────────────────────────────────────────────────────────────

pub struct UrlUtil;

impl UrlUtil {
    #[inline]
    pub fn parse(url_str: &str) -> anyhow::Result<Url> {
        Url::parse(url_str).map_err(|e| anyhow::anyhow!("Invalid URL '{url_str}': {e}"))
    }

    #[inline]
    pub fn join(base: &Url, path: &str) -> anyhow::Result<Url> {
        base.join(path).map_err(|e| anyhow::anyhow!("Failed to join URL: {e}"))
    }

    #[inline]
    pub fn is_http(url: &Url) -> bool {
        matches!(url.scheme(), "http" | "https")
    }

    #[inline]
    pub fn is_localhost(url: &Url) -> bool {
        url.host_str().map_or(false, |h| h == "localhost" || h == "127.0.0.1" || h == "::1")
    }

    #[inline]
    pub fn query_param(url: &Url, key: &str) -> Option<String> {
        url.query_pairs().find(|(k, _)| k == key).map(|(_, v)| v.to_string())
    }
}

// ── JsonUtil ──────────────────────────────────────────────────────────────

pub struct JsonUtil;

impl JsonUtil {
    #[inline]
    pub fn to_string<T: Serialize>(value: &T) -> anyhow::Result<String> {
        serde_json::to_string(value).map_err(|e| anyhow::anyhow!("Serialization failed: {e}"))
    }

    #[inline]
    pub fn to_string_pretty<T: Serialize>(value: &T) -> anyhow::Result<String> {
        serde_json::to_string_pretty(value).map_err(|e| anyhow::anyhow!("Serialization failed: {e}"))
    }

    #[inline]
    pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> anyhow::Result<T> {
        serde_json::from_str(s).map_err(|e| anyhow::anyhow!("Deserialization failed: {e}"))
    }

    #[inline]
    pub fn to_vec<T: Serialize>(value: &T) -> anyhow::Result<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| anyhow::anyhow!("Serialization failed: {e}"))
    }

    #[inline]
    pub fn from_slice<'a, T: Deserialize<'a>>(data: &'a [u8]) -> anyhow::Result<T> {
        serde_json::from_slice(data).map_err(|e| anyhow::anyhow!("Deserialization failed: {e}"))
    }

    #[inline]
    pub fn is_valid(s: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(s).is_ok()
    }

    #[inline]
    pub fn merge(base: &mut serde_json::Value, overrides: serde_json::Value) {
        merge_json(base, overrides);
    }
}

fn merge_json(base: &mut serde_json::Value, overrides: serde_json::Value) {
    match (base, overrides) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(over_map)) => {
            for (k, v) in over_map {
                if let Some(existing) = base_map.get_mut(&k) {
                    merge_json(existing, v);
                } else {
                    base_map.insert(k, v);
                }
            }
        }
        (base, over) => *base = over,
    }
}

// ── TimeUtil ──────────────────────────────────────────────────────────────

pub struct TimeUtil;

impl TimeUtil {
    #[inline]
    pub fn format_duration(dur: &Duration) -> String {
        let total_ns = dur.as_nanos();
        if total_ns < 1_000 {
            return format!("{total_ns}ns");
        }
        if total_ns < 1_000_000 {
            return format!("{}µs", total_ns / 1_000);
        }
        if total_ns < 1_000_000_000 {
            return format!("{}ms", total_ns / 1_000_000);
        }
        let secs = dur.as_secs_f64();
        if secs < 60.0 {
            return format!("{secs:.2}s");
        }
        let mins = secs / 60.0;
        if mins < 60.0 {
            return format!("{mins:.2}m");
        }
        let hours = mins / 60.0;
        format!("{hours:.2}h")
    }

    #[inline]
    pub fn now_iso() -> String {
        Utc::now().to_rfc3339()
    }

    #[inline]
    pub fn unix_ts() -> i64 {
        Utc::now().timestamp()
    }

    #[inline]
    pub fn unix_ts_ms() -> i64 {
        Utc::now().timestamp_millis()
    }

    #[inline]
    pub fn parse_iso(s: &str) -> anyhow::Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| s.parse::<DateTime<Utc>>())
            .map_err(|e| anyhow::anyhow!("Failed to parse datetime '{s}': {e}"))
    }
}

// ── StrUtil ───────────────────────────────────────────────────────────────

pub struct StrUtil;

impl StrUtil {
    #[inline]
    pub fn to_kebab(s: &str) -> String {
        s.trim()
            .chars()
            .flat_map(|c| {
                if c.is_uppercase() {
                    vec!['-', c.to_ascii_lowercase()]
                } else if c.is_alphanumeric() || c == '-' {
                    vec![c]
                } else {
                    vec!['-']
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }

    #[inline]
    pub fn to_snake(s: &str) -> String {
        s.trim()
            .chars()
            .flat_map(|c| {
                if c.is_uppercase() {
                    vec!['_', c.to_ascii_lowercase()]
                } else if c.is_alphanumeric() || c == '_' {
                    vec![c]
                } else {
                    vec!['_']
                }
            })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }

    #[inline]
    pub fn to_camel(s: &str) -> String {
        s.trim()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .enumerate()
            .map(|(i, w)| {
                if i == 0 {
                    w.to_lowercase()
                } else {
                    let mut chars = w.chars();
                    match chars.next() {
                        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                        None => String::new(),
                    }
                }
            })
            .collect()
    }

    #[inline]
    pub fn to_pascal(s: &str) -> String {
        s.trim()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| !w.is_empty())
            .map(|w| {
                let mut chars = w.chars();
                match chars.next() {
                    Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect()
    }

    #[inline]
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            let mut end = max_len.saturating_sub(3);
            while end > 0 && !s.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}...", &s[..end])
        }
    }

    pub fn slug(s: &str) -> String {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^a-zA-Z0-9\-_]").unwrap());
        RE.replace_all(
            &s.to_lowercase().replace(' ', "-").replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-"),
            ""
        ).to_string()
    }

    #[inline]
    pub fn indent(s: &str, level: usize) -> String {
        let prefix = "  ".repeat(level);
        s.lines().map(|line| format!("{prefix}{line}")).collect::<Vec<_>>().join("\n")
    }

    #[inline]
    pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
        if count == 1 { singular.to_string() } else { plural.to_string() }
    }
}

// ── Legacy re-exports (backward compat) ───────────────────────────────────

pub fn is_valid_semver(version: &str) -> bool {
    Semver::parse(version).is_ok()
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
    HashUtil::sha256(input.as_bytes())
}

pub fn temp_dir() -> PathBuf {
    std::env::temp_dir().join("klyron")
}

pub fn ensure_temp_dir() -> anyhow::Result<PathBuf> {
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
    StrUtil::slug(name)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let v = Semver::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre.is_none());
        assert!(v.build.is_none());
    }

    #[test]
    fn test_semver_prerelease() {
        let v = Semver::parse("2.0.0-beta.1").unwrap();
        assert_eq!(v.pre, Some("beta.1".into()));
    }

    #[test]
    fn test_semver_build() {
        let v = Semver::parse("1.0.0+20210101").unwrap();
        assert_eq!(v.build, Some("20210101".into()));
    }

    #[test]
    fn test_semver_invalid() {
        assert!(Semver::parse("1.2").is_err());
        assert!(Semver::parse("abc").is_err());
    }

    #[test]
    fn test_semver_compatible() {
        let a = Semver::new(1, 2, 0);
        let b = Semver::new(1, 3, 0);
        assert!(b.compatible(&a));
        assert!(!a.compatible(&b));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("My-App_v2"), "my-app_v2");
    }

    #[test]
    fn test_hash() {
        let h = HashUtil::sha256(b"hello");
        assert_eq!(h.len(), 64);
        let h32 = HashUtil::xxhash32(b"hello");
        assert!(h32 > 0);
        let md = HashUtil::md5(b"hello");
        assert_eq!(md.len(), 32);
    }

    #[test]
    fn test_str_util() {
        assert_eq!(StrUtil::to_kebab("HelloWorld"), "hello-world");
        assert_eq!(StrUtil::to_snake("HelloWorld"), "hello_world");
        assert_eq!(StrUtil::to_camel("hello-world"), "helloWorld");
        assert_eq!(StrUtil::to_pascal("hello-world"), "HelloWorld");
        assert_eq!(StrUtil::truncate("hello world", 5), "he...");
        assert_eq!(StrUtil::pluralize(1, "item", "items"), "item");
        assert_eq!(StrUtil::pluralize(2, "item", "items"), "items");
    }

    #[test]
    fn test_time_util() {
        let dur = Duration::from_secs(5);
        let formatted = TimeUtil::format_duration(&dur);
        assert!(formatted.contains("5.00s") || formatted.contains("5s"));
        let iso = TimeUtil::now_iso();
        assert!(iso.contains('T'));
    }

    #[test]
    fn test_path_util() {
        assert_eq!(PathUtil::extension(Path::new("test.js")), "js");
        assert!(PathUtil::is_js_like(Path::new("test.tsx")));
        assert!(!PathUtil::is_js_like(Path::new("test.py")));
    }

    #[test]
    fn test_json_util() {
        let val = serde_json::json!({"a": 1});
        let s = JsonUtil::to_string(&val).unwrap();
        assert_eq!(s, r#"{"a":1}"#);
        assert!(JsonUtil::is_valid(r#"{"b":2}"#));
        assert!(!JsonUtil::is_valid("not json"));
    }

    #[test]
    fn test_is_valid_semver() {
        assert!(is_valid_semver("1.2.3"));
        assert!(!is_valid_semver("1.2"));
        assert!(!is_valid_semver("abc"));
    }
}
