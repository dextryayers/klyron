use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

pub trait DoctorCheck: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self) -> DoctorResult;
    fn severity(&self) -> Severity;
}

#[derive(Debug, Clone)]
pub enum DoctorResult {
    Pass,
    Warning(String),
    Error(String),
    Info(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

pub struct CheckResult {
    pub check: String,
    pub description: String,
    pub severity: Severity,
    pub result: DoctorResult,
    pub duration_ms: u64,
    pub suggestion: Option<String>,
}

pub enum OutputFormat {
    Text,
    Json,
    Pretty,
}

pub struct DoctorEngine {
    checks: Vec<Box<dyn DoctorCheck>>,
}

impl DoctorEngine {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn add_check(&mut self, check: Box<dyn DoctorCheck>) {
        self.checks.push(check);
    }

    pub fn run_all(&self) -> Vec<CheckResult> {
        self.checks.iter().map(|c| self.run_check(c.as_ref())).collect()
    }

    pub fn run_category(&self, _category: &str) -> Vec<CheckResult> {
        self.checks.iter().map(|c| self.run_check(c.as_ref())).collect()
    }

    fn run_check(&self, check: &dyn DoctorCheck) -> CheckResult {
        let start = Instant::now();
        let result = check.run();
        let duration_ms = start.elapsed().as_millis() as u64;
        let suggestion = match &result {
            DoctorResult::Error(msg) => Some(format!("Suggestion for '{}': {}", check.name(), msg)),
            DoctorResult::Warning(msg) => Some(format!("Recommendation: {msg}")),
            _ => None,
        };
        CheckResult {
            check: check.name().to_string(),
            description: check.description().to_string(),
            severity: check.severity(),
            result,
            duration_ms,
            suggestion,
        }
    }

    pub fn generate_report(&self, format: OutputFormat) -> String {
        let results = self.run_all();
        match format {
            OutputFormat::Json => {
                let items: Vec<serde_json::Value> = results.iter().map(|r| {
                    serde_json::json!({
                        "check": r.check,
                        "description": r.description,
                        "severity": format!("{:?}", r.severity),
                        "status": match &r.result {
                            DoctorResult::Pass => "pass",
                            DoctorResult::Warning(_) => "warning",
                            DoctorResult::Error(_) => "error",
                            DoctorResult::Info(_) => "info",
                        },
                        "message": match &r.result {
                            DoctorResult::Pass => String::new(),
                            DoctorResult::Warning(m) => m.clone(),
                            DoctorResult::Error(m) => m.clone(),
                            DoctorResult::Info(m) => m.clone(),
                        },
                        "duration_ms": r.duration_ms,
                        "suggestion": r.suggestion,
                    })
                }).collect();
                serde_json::to_string_pretty(&serde_json::json!({
                    "diagnostics": items,
                    "timestamp": chrono_now_iso(),
                })).unwrap_or_default()
            }
            OutputFormat::Text | OutputFormat::Pretty => {
                let mut out = String::new();
                out.push_str(&format!("Klyron Diagnostics Report\n{}\n\n", "=".repeat(50)));
                let mut passed = 0u32;
                let mut warnings = 0u32;
                let mut errors = 0u32;
                for r in &results {
                    let icon = match &r.result {
                        DoctorResult::Pass => { passed += 1; "\u{2705}" }
                        DoctorResult::Info(_) => { passed += 1; "\u{2139}\u{FE0F}" }
                        DoctorResult::Warning(_) => { warnings += 1; "\u{26A0}\u{FE0F}" }
                        DoctorResult::Error(_) => { errors += 1; "\u{274C}" }
                    };
                    let sev = format!("{:?}", r.severity);
                    out.push_str(&format!(" {icon} [{sev:>8}] {} ({})\n", r.check, r.description));
                    if let DoctorResult::Error(m) | DoctorResult::Warning(m) | DoctorResult::Info(m) = &r.result {
                        out.push_str(&format!("       {m}\n"));
                    }
                    if let Some(s) = &r.suggestion {
                        out.push_str(&format!("       \u{1F4A1} {s}\n"));
                    }
                    out.push_str(&format!("       \u{23F1} {}ms\n", r.duration_ms));
                }
                out.push_str(&format!("\nSummary: {passed} passed, {warnings} warnings, {errors} errors\n"));
                out
            }
        }
    }

    pub fn register_all(&mut self) {
        self.add_check(Box::new(SystemCheck));
        self.add_check(Box::new(NetworkCheck));
        self.add_check(Box::new(RuntimeCheck));
        self.add_check(Box::new(ProjectCheck));
        self.add_check(Box::new(PermissionCheck));
        self.add_check(Box::new(DockerCheck));
        self.add_check(Box::new(NodeCompatCheck));
        self.add_check(Box::new(PortCheck));
    }
}

impl Default for DoctorEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let millis = d.subsec_millis();
    let days = secs / 86400;
    let time = secs % 86400;
    let hours = time / 3600;
    let mins = (time % 3600) / 60;
    let secs = time % 60;
    format!("{days:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{millis:03}Z", 1 + (days % 31), 1 + (days % 12), hours, mins, secs)
}

pub struct SystemCheck;

impl DoctorCheck for SystemCheck {
    fn name(&self) -> &'static str { "system" }
    fn description(&self) -> &'static str { "OS, CPU, memory and disk check" }
    fn severity(&self) -> Severity { Severity::Info }

    fn run(&self) -> DoctorResult {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0);
        let mem_kb = read_vm_rss().unwrap_or(0);
        let mem_mb = mem_kb / 1024;

        let disk_info = match dirs::cache_dir() {
            Some(dir) => {
                let avail = fs_available_space(&dir).unwrap_or(0);
                let avail_mb = avail / (1024 * 1024);
                format!(", disk: {avail_mb}MB free")
            }
            None => String::new(),
        };

        DoctorResult::Info(format!("{os}/{arch}, {cpus} cores, ~{mem_mb}MB RAM{disk_info}"))
    }
}

pub struct NetworkCheck;

impl DoctorCheck for NetworkCheck {
    fn name(&self) -> &'static str { "network" }
    fn description(&self) -> &'static str { "Network connectivity check" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn run(&self) -> DoctorResult {
        let checks: &[(&str, &str)] = &[
            ("npm registry", "https://registry.npmjs.org/"),
            ("GitHub", "https://github.com/"),
            ("Google DNS", "https://dns.google/resolve?name=example.com"),
        ];

        let mut failures = Vec::new();
        for (label, url) in checks {
            match ureq::get(url).timeout(std::time::Duration::from_secs(5)).call() {
                Ok(resp) if resp.status() < 500 => {}
                _ => { failures.push(*label); }
            }
        }

        if failures.is_empty() {
            DoctorResult::Pass
        } else {
            DoctorResult::Warning(format!("Cannot reach: {}", failures.join(", ")))
        }
    }
}

pub struct RuntimeCheck;

impl DoctorCheck for RuntimeCheck {
    fn name(&self) -> &'static str { "runtime" }
    fn description(&self) -> &'static str { "Runtime version and engine availability" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn run(&self) -> DoctorResult {
        let version = env!("CARGO_PKG_VERSION");
        let mut engines = Vec::new();

        for kind in &["v8", "boa", "quickjs", "jsc"] {
            let is_avail = check_engine(kind);
            engines.push(format!("{kind}:{}", if is_avail { "available" } else { "unavailable" }));
        }

        DoctorResult::Info(format!("klyron v{version}, engines: {}", engines.join(", ")))
    }
}

fn check_engine(_name: &str) -> bool {
    true
}

pub struct ProjectCheck;

impl DoctorCheck for ProjectCheck {
    fn name(&self) -> &'static str { "project" }
    fn description(&self) -> &'static str { "Project configuration validation" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn run(&self) -> DoctorResult {
        let cwd = std::env::current_dir().unwrap_or_default();
        let mut issues = Vec::new();

        if !cwd.join("package.json").exists() {
            issues.push("No package.json found");
        }
        if !cwd.join("klyron.json").exists() && !cwd.join("klyron.config.json").exists() {
            issues.push("No klyron config file found");
        }
        if !cwd.join(".git").exists() {
            issues.push("Not a git repository");
        }
        if !cwd.join("node_modules").exists() {
            issues.push("node_modules not found (dependencies not installed)");
        }

        if issues.is_empty() {
            DoctorResult::Pass
        } else {
            DoctorResult::Warning(issues.join("; "))
        }
    }
}

pub struct PermissionCheck;

impl DoctorCheck for PermissionCheck {
    fn name(&self) -> &'static str { "permissions" }
    fn description(&self) -> &'static str { "File system and port permission check" }
    fn severity(&self) -> Severity { Severity::Critical }

    fn run(&self) -> DoctorResult {
        let mut issues = Vec::new();

        let test_dir = std::env::temp_dir().join("klyron_perm_test");
        match std::fs::create_dir_all(&test_dir) {
            Ok(_) => {
                let test_file = test_dir.join("test_write");
                match std::fs::write(&test_file, b"test") {
                    Ok(_) => { let _ = std::fs::remove_file(&test_file); }
                    Err(_) => issues.push("Cannot write to filesystem");
                }
                let _ = std::fs::remove_dir_all(&test_dir);
            }
            Err(_) => issues.push("Cannot create temp directory");
        }

        if issues.is_empty() {
            DoctorResult::Pass
        } else {
            DoctorResult::Error(issues.join("; "))
        }
    }
}

pub struct DockerCheck;

impl DoctorCheck for DockerCheck {
    fn name(&self) -> &'static str { "docker" }
    fn description(&self) -> &'static str { "Docker availability check" }
    fn severity(&self) -> Severity { Severity::Info }

    fn run(&self) -> DoctorResult {
        match std::process::Command::new("docker")
            .arg("--version")
            .output()
        {
            Ok(output) if output.status.success() => {
                let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                DoctorResult::Info(format!("Docker available: {ver}"))
            }
            _ => DoctorResult::Info("Docker not found in PATH".into()),
        }
    }
}

pub struct NodeCompatCheck;

impl DoctorCheck for NodeCompatCheck {
    fn name(&self) -> &'static str { "node_compat" }
    fn description(&self) -> &'static str { "Node.js version compatibility" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn run(&self) -> DoctorResult {
        match std::process::Command::new("node")
            .arg("--version")
            .output()
        {
            Ok(output) if output.status.success() => {
                let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let ver_num = ver.trim_start_matches('v').split('.').next().unwrap_or("0");
                let major: u32 = ver_num.parse().unwrap_or(0);
                if major >= 18 {
                    DoctorResult::Info(format!("Node.js {ver} (compatible)"))
                } else if major >= 14 {
                    DoctorResult::Warning(format!("Node.js {ver} (minimum recommended: 18)"))
                } else {
                    DoctorResult::Error(format!("Node.js {ver} is too old. Please upgrade to v18+"))
                }
            }
            _ => DoctorResult::Error("Node.js not found in PATH".into()),
        }
    }
}

pub struct PortCheck;

impl DoctorCheck for PortCheck {
    fn name(&self) -> &'static str { "ports" }
    fn description(&self) -> &'static str { "Common port availability check" }
    fn severity(&self) -> Severity { Severity::Warning }

    fn run(&self) -> DoctorResult {
        let common_ports = [3000, 3001, 5173, 8080, 8000, 4200, 5000];
        let mut occupied = Vec::new();

        for port in &common_ports {
            if is_port_in_use(*port) {
                occupied.push(port.to_string());
            }
        }

        if occupied.is_empty() {
            DoctorResult::Pass
        } else {
            DoctorResult::Warning(format!("Ports in use: {}", occupied.join(", ")))
        }
    }
}

fn is_port_in_use(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(format!("127.0.0.1:{port}")).is_err()
}

fn read_vm_rss() -> Result<u64, String> {
    let content = std::fs::read_to_string("/proc/self/status").map_err(|e| e.to_string())?;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("VmRSS:") {
            let v = value.trim().trim_end_matches(" kB");
            return v.parse::<u64>().map_err(|e| e.to_string());
        }
    }
    Ok(0)
}

fn fs_available_space(path: &std::path::Path) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let stat = std::fs::metadata(path).ok()?;
        let dev = stat.dev();
        let content = std::fs::read_to_string("/proc/mounts").ok()?;
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let mount_point = parts[1];
                let mount_stat = std::fs::metadata(mount_point).ok()?;
                if mount_stat.dev() == dev {
                    return statvfs_available(mount_point);
                }
            }
        }
        None
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

#[cfg(unix)]
fn statvfs_available(path: &str) -> Option<u64> {
    use std::ffi::CString;
    let cpath = CString::new(path).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(cpath.as_ptr(), &mut stat) } == 0 {
        Some(stat.f_frsize as u64 * stat.f_bavail as u64)
    } else {
        None
    }
}
