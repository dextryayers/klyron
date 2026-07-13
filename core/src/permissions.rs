use std::{
  fmt,
  net::ToSocketAddrs,
  str::FromStr,
  sync::{Arc, Mutex},
  time::SystemTime,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyTemplate {
  Default,
  Strict,
  Laravel,
  NextJs,
  Django,
  Rails,
}

impl PolicyTemplate {
  pub fn apply(&self) -> PermissionSet {
    match self {
      Self::Default => PermissionSet {
        allow_read_all: true,
        allow_write_all: true,
        allow_net_all: true,
        allow_env_all: true,
        sandbox: SandboxLevel::Basic,
        ..Default::default()
      },
      Self::Strict => PermissionSet {
        sandbox: SandboxLevel::Maximum,
        ..Default::default()
      },
      Self::Laravel => PermissionSet {
        allow_read: vec!["/app/**".to_string(), "/project/**".to_string()],
        allow_write: vec!["/app/storage/**".to_string(), "/app/bootstrap/cache/**".to_string()],
        allow_net: vec!["localhost:*".to_string()],
        allow_env: vec!["APP_*".to_string(), "DB_*".to_string(), "REDIS_*".to_string()],
        allow_run: vec!["php".to_string()],
        sandbox: SandboxLevel::Strict,
        ..Default::default()
      },
      Self::NextJs => PermissionSet {
        allow_read: vec!["/**".to_string()],
        allow_write: vec!["/.next/**".to_string(), "/node_modules/**".to_string()],
        allow_net: vec!["*".to_string()],
        allow_env: vec!["NODE_*".to_string(), "NEXT_*".to_string()],
        sandbox: SandboxLevel::Basic,
        ..Default::default()
      },
      Self::Django => PermissionSet {
        allow_read: vec!["/app/**".to_string()],
        allow_write: vec!["/app/db.sqlite3".to_string(), "/app/media/**".to_string(), "/app/static/**".to_string()],
        allow_net: vec!["localhost:*".to_string()],
        allow_env: vec!["DJANGO_*".to_string(), "DATABASE_*".to_string(), "SECRET_*".to_string()],
        sandbox: SandboxLevel::Strict,
        ..Default::default()
      },
      Self::Rails => PermissionSet {
        allow_read: vec!["/app/**".to_string()],
        allow_write: vec!["/app/log/**".to_string(), "/app/tmp/**".to_string(), "/app/storage/**".to_string()],
        allow_net: vec!["localhost:*".to_string()],
        allow_env: vec!["RAILS_*".to_string(), "DATABASE_*".to_string()],
        allow_run: vec!["ruby".to_string(), "rake".to_string(), "rails".to_string()],
        sandbox: SandboxLevel::Strict,
        ..Default::default()
      },
    }
  }

  pub fn variants() -> &'static [&'static str] {
    &["default", "strict", "laravel", "nextjs", "django", "rails"]
  }
}

impl FromStr for PolicyTemplate {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "default" => Ok(Self::Default),
      "strict" => Ok(Self::Strict),
      "laravel" => Ok(Self::Laravel),
      "nextjs" | "next.js" | "next" => Ok(Self::NextJs),
      "django" => Ok(Self::Django),
      "rails" | "ruby" => Ok(Self::Rails),
      _ => Err(format!("Unknown policy template: {s}. Choose: {}", Self::variants().join(", "))),
    }
  }
}

impl fmt::Display for PolicyTemplate {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Default => write!(f, "default"),
      Self::Strict => write!(f, "strict"),
      Self::Laravel => write!(f, "laravel"),
      Self::NextJs => write!(f, "nextjs"),
      Self::Django => write!(f, "django"),
      Self::Rails => write!(f, "rails"),
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct PermissionSet {
  pub allow_read: Vec<String>,
  pub deny_read: Vec<String>,
  pub allow_write: Vec<String>,
  pub deny_write: Vec<String>,
  pub allow_net: Vec<String>,
  pub deny_net: Vec<String>,
  pub allow_env: Vec<String>,
  pub deny_env: Vec<String>,
  pub allow_run: Vec<String>,
  pub allow_ffi: bool,
  pub allow_sys: bool,
  pub allow_read_all: bool,
  pub allow_write_all: bool,
  pub allow_net_all: bool,
  pub allow_env_all: bool,
  pub prompt: bool,
  pub sandbox: SandboxLevel,
  pub max_memory: Option<u64>,
  pub max_cpu: Option<u64>,
  pub max_fds: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SandboxLevel {
  #[default]
  None,
  Basic,
  Strict,
  Maximum,
}

impl SandboxLevel {
  pub fn is_sandboxed(&self) -> bool {
    matches!(self, Self::Basic | Self::Strict | Self::Maximum)
  }
}

impl FromStr for SandboxLevel {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "none" => Ok(Self::None),
      "basic" => Ok(Self::Basic),
      "strict" => Ok(Self::Strict),
      "maximum" => Ok(Self::Maximum),
      _ => Err(format!("Invalid sandbox level: {s}. Choose: none, basic, strict, maximum")),
    }
  }
}

impl fmt::Display for SandboxLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::None => write!(f, "none"),
      Self::Basic => write!(f, "basic"),
      Self::Strict => write!(f, "strict"),
      Self::Maximum => write!(f, "maximum"),
    }
  }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuditEntry {
  pub timestamp: String,
  pub pid: u32,
  pub operation: String,
  pub resource: String,
  pub allowed: bool,
  pub rule: Option<String>,
}

pub struct Permissions {
  pub set: PermissionSet,
  resolved_read: Vec<glob::Pattern>,
  resolved_write: Vec<glob::Pattern>,
  resolved_net: Vec<glob::Pattern>,
  resolved_deny_read: Vec<glob::Pattern>,
  resolved_deny_write: Vec<glob::Pattern>,
  resolved_deny_net: Vec<glob::Pattern>,
  resolved_deny_env: Vec<glob::Pattern>,
  resolved_env: Vec<glob::Pattern>,
  resolved_run: Vec<glob::Pattern>,
  pub audit_log: Arc<Mutex<Vec<AuditEntry>>>,
}

fn compile_patterns(patterns: &[String]) -> Vec<glob::Pattern> {
  let mut out = Vec::new();
  for p in patterns {
    if p.contains('*') || p.contains('?') || p.contains('[') {
      if let Ok(pat) = glob::Pattern::new(p) {
        out.push(pat);
      }
    } else {
      // Plain path: match exact + everything underneath
      let exact = p.trim_end_matches('/');
      if let Ok(pat) = glob::Pattern::new(exact) {
        out.push(pat);
      }
      if let Ok(pat) = glob::Pattern::new(&format!("{exact}/**")) {
        out.push(pat);
      }
    }
  }
  out
}

fn compile_exact_patterns(patterns: &[String]) -> Vec<glob::Pattern> {
  patterns
    .iter()
    .filter_map(|p| glob::Pattern::new(p).ok())
    .collect()
}

impl Permissions {
  pub fn new(set: PermissionSet) -> Self {
    let resolved_read = compile_patterns(&set.allow_read);
    let resolved_write = compile_patterns(&set.allow_write);
    let resolved_net = compile_patterns(&set.allow_net);
    let resolved_deny_read = compile_patterns(&set.deny_read);
    let resolved_deny_write = compile_patterns(&set.deny_write);
    let resolved_deny_net = compile_patterns(&set.deny_net);
    let resolved_deny_env = compile_patterns(&set.deny_env);
    let resolved_env = compile_patterns(&set.allow_env);
    let resolved_run = compile_exact_patterns(&set.allow_run);

    Self {
      set,
      resolved_read,
      resolved_write,
      resolved_net,
      resolved_deny_read,
      resolved_deny_write,
      resolved_deny_net,
      resolved_deny_env,
      resolved_env,
      resolved_run,
      audit_log: Arc::new(Mutex::new(Vec::new())),
    }
  }

  pub fn check_read(&self, path: &str) -> Result<(), String> {
    // Deny always takes precedence
    for pat in &self.resolved_deny_read {
      if pat.matches(path) {
        self.log("read", path, false, &format!("deny_read: {pat}"));
        return Err(format!("Read access denied: {path}"));
      }
    }
    if self.set.allow_read_all {
      self.log("read", path, true, "allow_read_all");
      return Ok(());
    }
    for pat in &self.resolved_read {
      if pat.matches(path) {
        self.log("read", path, true, &format!("allow_read: {pat}"));
        return Ok(());
      }
    }
    if self.set.prompt {
      return self.prompt_user("read", path);
    }
    self.log("read", path, false, "implicit_deny");
    Err(format!("Read access denied: {path}"))
  }

  pub fn check_write(&self, path: &str) -> Result<(), String> {
    for pat in &self.resolved_deny_write {
      if pat.matches(path) {
        self.log("write", path, false, &format!("deny_write: {pat}"));
        return Err(format!("Write access denied: {path}"));
      }
    }
    if self.set.allow_write_all {
      self.log("write", path, true, "allow_write_all");
      return Ok(());
    }
    for pat in &self.resolved_write {
      if pat.matches(path) {
        self.log("write", path, true, &format!("allow_write: {pat}"));
        return Ok(());
      }
    }
    if self.set.prompt {
      return self.prompt_user("write", path);
    }
    self.log("write", path, false, "implicit_deny");
    Err(format!("Write access denied: {path}"))
  }

  pub fn check_net(&self, host: &str) -> Result<(), String> {
    if self.check_net_impl(host) {
      return Ok(());
    }
    // DNS-based resolution: try resolving hostname to IPs
    if let Some(resolved) = self.resolve_net(host) {
      if self.check_net_impl(&resolved) {
        self.log("net", host, true, &format!("dns_resolved: {resolved}"));
        return Ok(());
      }
    }
    if self.set.prompt {
      return self.prompt_user("net", host);
    }
    self.log("net", host, false, "implicit_deny");
    Err(format!("Network access denied: {host}"))
  }

  fn check_net_impl(&self, host: &str) -> bool {
    let hostname = host.split(':').next().unwrap_or(host);
    for pat in &self.resolved_deny_net {
      if pat.matches(hostname) || pat.matches(host) {
        self.log("net", host, false, &format!("deny_net: {pat}"));
        return false;
      }
    }
    if self.set.allow_net_all {
      self.log("net", host, true, "allow_net_all");
      return true;
    }
    for pat in &self.resolved_net {
      if pat.matches(hostname) || pat.matches(host) {
        self.log("net", host, true, &format!("allow_net: {pat}"));
        return true;
      }
    }
    false
  }

  fn resolve_net(&self, host: &str) -> Option<String> {
    let (hostname, port) = if let Some(idx) = host.rfind(':') {
      (&host[..idx], &host[idx + 1..])
    } else {
      (host, "0")
    };
    if hostname.contains('.') && !hostname.chars().all(|c| c.is_ascii_digit() || c == '.') {
      if let Ok(addrs) = (hostname, port.parse::<u16>().unwrap_or(0)).to_socket_addrs() {
        for addr in addrs {
          let ip_str = addr.ip().to_string();
          for pat in &self.resolved_deny_net {
            if pat.matches(&ip_str) || pat.matches(&format!("{ip_str}:{port}")) {
              return None;
            }
          }
          if self.set.allow_net_all {
            return Some(format!("{ip_str}:{port}"));
          }
          for pat in &self.resolved_net {
            if pat.matches(&ip_str) || pat.matches(&format!("{ip_str}:{port}")) {
              return Some(format!("{ip_str}:{port}"));
            }
          }
        }
      }
    }
    None
  }

  pub fn check_env(&self, name: &str) -> Result<(), String> {
    for pat in &self.resolved_deny_env {
      if pat.matches(name) {
        self.log("env", name, false, &format!("deny_env: {pat}"));
        return Err(format!("Environment access denied: {name}"));
      }
    }
    if self.set.allow_env_all {
      self.log("env", name, true, "allow_env_all");
      return Ok(());
    }
    for pat in &self.resolved_env {
      if pat.matches(name) {
        self.log("env", name, true, &format!("allow_env: {pat}"));
        return Ok(());
      }
    }
    if self.set.prompt {
      return self.prompt_user("env", name);
    }
    self.log("env", name, false, "implicit_deny");
    Err(format!("Environment access denied: {name}"))
  }

  pub fn check_run(&self, cmd: &str) -> Result<(), String> {
    if self.set.allow_run.is_empty() {
      self.log("run", cmd, false, "no_allow_run");
      return Err(format!("Subprocess execution denied: {cmd}"));
    }
    for pat in &self.resolved_run {
      if pat.matches(cmd) {
        self.log("run", cmd, true, &format!("allow_run: {pat}"));
        return Ok(());
      }
    }
    if self.set.prompt {
      return self.prompt_user("run", cmd);
    }
    self.log("run", cmd, false, "implicit_deny");
    Err(format!("Subprocess execution denied: {cmd}"))
  }

  /// Create an inherited PermissionSet for a worker/child process.
  /// Children inherit the intersection of the parent's permissions.
  /// If the parent has allow_read_all, the child also gets allow_read_all.
  /// Specific allow lists are inherited as-is; empty lists remain empty.
  pub fn inherited_set(&self) -> PermissionSet {
    PermissionSet {
      allow_read: self.set.allow_read.clone(),
      deny_read: self.set.deny_read.clone(),
      allow_write: self.set.allow_write.clone(),
      deny_write: self.set.deny_write.clone(),
      allow_net: self.set.allow_net.clone(),
      deny_net: self.set.deny_net.clone(),
      allow_env: self.set.allow_env.clone(),
      deny_env: self.set.deny_env.clone(),
      allow_run: self.set.allow_run.clone(),
      allow_ffi: self.set.allow_ffi,
      allow_sys: self.set.allow_sys,
      allow_read_all: self.set.allow_read_all,
      allow_write_all: self.set.allow_write_all,
      allow_net_all: self.set.allow_net_all,
      allow_env_all: self.set.allow_env_all,
      prompt: self.set.prompt,
      sandbox: self.set.sandbox,
      max_memory: self.set.max_memory,
      max_cpu: self.set.max_cpu,
      max_fds: self.set.max_fds,
    }
  }

  pub fn drain_audit_log(&self) -> Vec<AuditEntry> {
    if let Ok(mut log) = self.audit_log.lock() {
      std::mem::take(&mut *log)
    } else {
      vec![]
    }
  }

  fn log(&self, operation: &str, resource: &str, allowed: bool, rule: &str) {
    if let Ok(mut log) = self.audit_log.lock() {
      let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
      log.push(AuditEntry {
        timestamp: format!("{:.6}", timestamp),
        pid: std::process::id(),
        operation: operation.to_string(),
        resource: resource.to_string(),
        allowed,
        rule: Some(rule.to_string()),
      });
    }
  }

  fn prompt_user(&self, operation: &str, resource: &str) -> Result<(), String> {
    eprintln!(
      "Klyron requires {} access to \"{}\". Allow? [y/N] ",
      operation, resource
    );
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
      if input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes") {
        self.log(operation, resource, true, "prompt_approved");
        return Ok(());
      }
    }
    self.log(operation, resource, false, "prompt_denied");
    Err(format!("{operation} access denied by user: {resource}"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn perms() -> Permissions {
    Permissions::new(PermissionSet::default())
  }

  fn perms_with(set: PermissionSet) -> Permissions {
    Permissions::new(set)
  }

  #[test]
  fn test_check_read_allow_all() {
    let p = perms_with(PermissionSet { allow_read_all: true, ..Default::default() });
    assert!(p.check_read("/etc/passwd").is_ok());
    assert!(p.check_read("/").is_ok());
  }

  #[test]
  fn test_check_read_deny_overrides_allow_all() {
    let p = perms_with(PermissionSet {
      allow_read_all: true,
      deny_read: vec!["/etc/**".to_string()],
      ..Default::default()
    });
    assert!(p.check_read("/home/user/file.txt").is_ok());
    assert!(p.check_read("/etc/passwd").is_err());
    assert!(p.check_read("/etc/shadow").is_err());
  }

  #[test]
  fn test_check_read_explicit_allow() {
    let p = perms_with(PermissionSet {
      allow_read: vec!["/app/**".to_string()],
      ..Default::default()
    });
    assert!(p.check_read("/app/src/main.ts").is_ok());
    assert!(p.check_read("/etc/passwd").is_err());
  }

  #[test]
  fn test_check_read_implicit_deny() {
    let p = perms();
    assert!(p.check_read("/any/file").is_err());
  }

  #[test]
  fn test_check_write_allow_all() {
    let p = perms_with(PermissionSet { allow_write_all: true, ..Default::default() });
    assert!(p.check_write("/tmp/foo").is_ok());
  }

  #[test]
  fn test_check_write_deny_overrides() {
    let p = perms_with(PermissionSet {
      allow_write_all: true,
      deny_write: vec!["/etc/**".to_string()],
      ..Default::default()
    });
    assert!(p.check_write("/tmp/foo").is_ok());
    assert!(p.check_write("/etc/hosts").is_err());
  }

  #[test]
  fn test_check_net_allow_all() {
    let p = perms_with(PermissionSet { allow_net_all: true, ..Default::default() });
    assert!(p.check_net("example.com:80").is_ok());
    assert!(p.check_net("192.168.1.1:443").is_ok());
  }

  #[test]
  fn test_check_net_deny_domain() {
    let p = perms_with(PermissionSet {
      allow_net_all: true,
      deny_net: vec!["*.malicious.com".to_string()],
      ..Default::default()
    });
    assert!(p.check_net("example.com:80").is_ok());
    assert!(p.check_net("evil.malicious.com:443").is_err());
  }

  #[test]
  fn test_check_env_allow_all() {
    let p = perms_with(PermissionSet { allow_env_all: true, ..Default::default() });
    assert!(p.check_env("PATH").is_ok());
    assert!(p.check_env("HOME").is_ok());
  }

  #[test]
  fn test_check_env_deny() {
    let p = perms_with(PermissionSet {
      allow_env_all: true,
      deny_env: vec!["SECRET_*".to_string()],
      ..Default::default()
    });
    assert!(p.check_env("PATH").is_ok());
    assert!(p.check_env("SECRET_KEY").is_err());
  }

  #[test]
  fn test_check_run_denied_by_default() {
    let p = perms();
    assert!(p.check_run("bash").is_err());
    assert!(p.check_run("node").is_err());
  }

  #[test]
  fn test_check_run_allowed() {
    let p = perms_with(PermissionSet {
      allow_run: vec!["node".to_string(), "npm".to_string()],
      ..Default::default()
    });
    assert!(p.check_run("node").is_ok());
    assert!(p.check_run("npm").is_ok());
    assert!(p.check_run("bash").is_err());
  }

  #[test]
  fn test_audit_log_read() {
    let p = perms();
    let _ = p.check_read("/secret.txt");
    let log = p.drain_audit_log();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].operation, "read");
    assert_eq!(log[0].resource, "/secret.txt");
    assert!(!log[0].allowed);
    assert_eq!(log[0].rule.as_deref(), Some("implicit_deny"));
  }

  #[test]
  fn test_audit_log_allowed() {
    let p = perms_with(PermissionSet { allow_read_all: true, ..Default::default() });
    let _ = p.check_read("/any/path");
    let log = p.drain_audit_log();
    assert_eq!(log.len(), 1);
    assert!(log[0].allowed);
    assert_eq!(log[0].rule.as_deref(), Some("allow_read_all"));
  }

  #[test]
  fn test_audit_log_drain_clears() {
    let p = perms_with(PermissionSet { allow_read_all: true, ..Default::default() });
    let _ = p.check_read("/a");
    assert_eq!(p.drain_audit_log().len(), 1);
    assert_eq!(p.drain_audit_log().len(), 0);
  }

  #[test]
  fn test_check_read_prefix_pattern() {
    let p = perms_with(PermissionSet {
      allow_read: vec!["/app".to_string()],
      ..Default::default()
    });
    assert!(p.check_read("/app/src/main.ts").is_ok());
    assert!(p.check_read("/app/").is_ok());
    assert!(p.check_read("/app").is_ok());
    assert!(p.check_read("/etc/passwd").is_err());
  }

  #[test]
  fn test_check_read_exact_file() {
    let p = perms_with(PermissionSet {
      allow_read: vec!["/app.ts".to_string()],
      ..Default::default()
    });
    assert!(p.check_read("/app.ts").is_ok());
    assert!(p.check_read("/other.ts").is_err());
  }

  #[test]
  fn test_sandbox_level_parse() {
    assert_eq!("none".parse::<SandboxLevel>().unwrap(), SandboxLevel::None);
    assert_eq!("basic".parse::<SandboxLevel>().unwrap(), SandboxLevel::Basic);
    assert_eq!("strict".parse::<SandboxLevel>().unwrap(), SandboxLevel::Strict);
    assert_eq!("maximum".parse::<SandboxLevel>().unwrap(), SandboxLevel::Maximum);
    assert!("invalid".parse::<SandboxLevel>().is_err());
  }

  #[test]
  fn test_sandbox_level_is_sandboxed() {
    assert!(!SandboxLevel::None.is_sandboxed());
    assert!(SandboxLevel::Basic.is_sandboxed());
    assert!(SandboxLevel::Strict.is_sandboxed());
    assert!(SandboxLevel::Maximum.is_sandboxed());
  }
}
