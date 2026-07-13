use std::sync::atomic::{AtomicBool, Ordering};

use crate::permissions::SandboxLevel;

static SANDBOX_APPLIED: AtomicBool = AtomicBool::new(false);

pub struct Sandbox;

impl Sandbox {
  pub fn apply(level: SandboxLevel, max_memory: Option<u64>, max_cpu: Option<u64>, max_fds: Option<u64>) -> Result<(), String> {
    if SANDBOX_APPLIED.swap(true, Ordering::SeqCst) {
      return Err("Sandbox already applied".to_string());
    }
    match level {
      SandboxLevel::None => Ok(()),
      _ => {
        apply_resource_limits(max_memory, max_cpu, max_fds)?;
        apply_os_sandbox(level)?;
        Ok(())
      }
    }
  }
}

fn apply_resource_limits(max_memory: Option<u64>, max_cpu: Option<u64>, max_fds: Option<u64>) -> Result<(), String> {
  unsafe {
    if let Some(mem_mb) = max_memory {
      let bytes = mem_mb * 1024 * 1024;
      let rlim = libc::rlimit { rlim_cur: bytes, rlim_max: bytes };
      if libc::setrlimit(libc::RLIMIT_AS, &rlim) != 0 {
        return Err(format!("Failed to set RLIMIT_AS: {}", std::io::Error::last_os_error()));
      }
    }
    if let Some(cpu_secs) = max_cpu {
      let rlim = libc::rlimit { rlim_cur: cpu_secs, rlim_max: cpu_secs };
      if libc::setrlimit(libc::RLIMIT_CPU, &rlim) != 0 {
        return Err(format!("Failed to set RLIMIT_CPU: {}", std::io::Error::last_os_error()));
      }
    }
    if let Some(fds) = max_fds {
      let rlim = libc::rlimit { rlim_cur: fds, rlim_max: fds };
      if libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) != 0 {
        return Err(format!("Failed to set RLIMIT_NOFILE: {}", std::io::Error::last_os_error()));
      }
    }
  }
  Ok(())
}

#[cfg(target_os = "linux")]
fn apply_os_sandbox(level: SandboxLevel) -> Result<(), String> {
  apply_namespace_isolation(level)?;
  apply_landlock(level)?;
  apply_seccomp_filter(level)?;
  Ok(())
}

#[cfg(not(target_os = "linux"))]
fn apply_os_sandbox(level: SandboxLevel) -> Result<(), String> {
  let _ = level;
  Err("Sandboxing is only supported on Linux".to_string())
}

// ─── Namespace Isolation ──────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn apply_namespace_isolation(level: SandboxLevel) -> Result<(), String> {
  if level == SandboxLevel::Maximum {
    try_unshare(libc::CLONE_NEWNS).ok();
    try_unshare(libc::CLONE_NEWIPC).ok();
  }
  Ok(())
}

#[cfg(target_os = "linux")]
fn try_unshare(flags: i32) -> Result<(), String> {
  let ret = unsafe { libc::unshare(flags) };
  if ret != 0 {
    let err = std::io::Error::last_os_error();
    if err.raw_os_error() == Some(libc::EPERM) {
      return Err(format!("unshare({flags:#x}) requires CAP_SYS_ADMIN: {err}"));
    }
    return Err(format!("unshare({flags:#x}) failed: {err}"));
  }
  Ok(())
}

// ─── Landlock LSM (Linux 5.13+) ───────────────────────────────────────────

#[cfg(target_os = "linux")]
fn apply_landlock(level: SandboxLevel) -> Result<(), String> {
  let abi = landlock_abi();
  if abi == 0 {
    return Err("Landlock LSM not available (requires Linux 5.13+ and CONFIG_SECURITY_LANDLOCK)".to_string());
  }

  let access_fs = match level {
    SandboxLevel::Basic => landlock_access_write() | landlock_access_create() | landlock_access_remove(),
    SandboxLevel::Strict => {
      landlock_access_write() | landlock_access_create() | landlock_access_remove()
    }
    SandboxLevel::Maximum => {
      landlock_access_read() | landlock_access_write() | landlock_access_create()
        | landlock_access_remove() | landlock_access_execute()
    }
    _ => return Ok(()),
  };

  let handled = if abi >= 1 { access_fs } else { 0 };

  let ruleset_fd = landlock_create_ruleset(handled)?;

  // Allow access to common runtime paths
  let allowed_paths = &[
    "/usr/lib",
    "/usr/lib64",
    "/lib",
    "/lib64",
    "/etc/ld.so.cache",
    "/etc/ld.so.conf",
    "/etc/ld.so.conf.d",
    "/etc/nsswitch.conf",
    "/etc/resolv.conf",
    "/etc/hosts",
    "/etc/host.conf",
    "/dev/urandom",
    "/dev/random",
    "/dev/null",
    "/dev/zero",
    "/proc/self",
    "/proc/sys",
    "/tmp",
    "/var/tmp",
  ];

  for path in allowed_paths {
    let allowed = match level {
      SandboxLevel::Basic | SandboxLevel::Strict => {
        landlock_access_read() | landlock_access_write() | landlock_access_execute()
      }
      SandboxLevel::Maximum => {
        landlock_access_read() | landlock_access_execute()
      }
      SandboxLevel::None => unreachable!(),
    };
    if let Err(e) = landlock_add_path_rule(ruleset_fd, allowed, path) {
      // Path may not exist, skip
      let _ = e;
    }
  }

  landlock_restrict_self(ruleset_fd)?;
  Ok(())
}

#[cfg(target_os = "linux")]
fn landlock_abi() -> u32 {
  unsafe {
    let ret = libc::syscall(libc::SYS_landlock_create_ruleset, std::ptr::null::<u8>(), 0, 0);
    if ret < 0 {
      return 0;
    }
    libc::close(ret as i32);
    1
  }
}

#[cfg(target_os = "linux")]
fn landlock_access_read() -> u64 {
  (1 << 2) | (1 << 3) // READ_FILE | READ_DIR
}

#[cfg(target_os = "linux")]
fn landlock_access_write() -> u64 {
  1 << 1 // WRITE_FILE
}

#[cfg(target_os = "linux")]
fn landlock_access_execute() -> u64 {
  1 << 0 // EXECUTE
}

#[cfg(target_os = "linux")]
fn landlock_access_create() -> u64 {
  (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10) | (1 << 11) | (1 << 12)
  // MAKE_DIR | MAKE_REG | MAKE_SOCK | MAKE_FIFO | MAKE_BLOCK | MAKE_SYM
}

#[cfg(target_os = "linux")]
fn landlock_access_remove() -> u64 {
  (1 << 4) | (1 << 5) // REMOVE_DIR | REMOVE_FILE
}

#[cfg(target_os = "linux")]
fn landlock_create_ruleset(handled_access_fs: u64) -> Result<i32, String> {
  #[repr(C)]
  struct landlock_ruleset_attr {
    handled_access_fs: u64,
  }

  let attr = landlock_ruleset_attr { handled_access_fs };
  unsafe {
    let ret = libc::syscall(
      libc::SYS_landlock_create_ruleset,
      &attr as *const _ as *const u8,
      std::mem::size_of::<landlock_ruleset_attr>(),
      0,
    );
    if ret < 0 {
      return Err(format!("landlock_create_ruleset failed: {}", std::io::Error::last_os_error()));
    }
    Ok(ret as i32)
  }
}

#[cfg(target_os = "linux")]
fn landlock_add_path_rule(ruleset_fd: i32, allowed_access: u64, path: &str) -> Result<(), String> {
  use std::ffi::CString;

  #[repr(C)]
  struct landlock_path_beneath_attr {
    allowed_access: u64,
    parent_fd: i32,
  }

  let cpath = CString::new(path).map_err(|e| format!("Invalid path {path}: {e}"))?;
  let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC | libc::O_PATH) };
  if fd < 0 {
    return Err(format!("Cannot open path {path}: {}", std::io::Error::last_os_error()));
  }

  let attr = landlock_path_beneath_attr { allowed_access, parent_fd: fd };
  unsafe {
    let ret = libc::syscall(
      libc::SYS_landlock_add_rule,
      ruleset_fd as i64,
      1, // LANDLOCK_RULE_PATH_BENEATH
      &attr as *const _ as i64,
      0,
    );
    libc::close(fd);
    if ret < 0 {
      return Err(format!("landlock_add_rule failed for {path}: {}", std::io::Error::last_os_error()));
    }
  }
  Ok(())
}

#[cfg(target_os = "linux")]
fn landlock_restrict_self(ruleset_fd: i32) -> Result<(), String> {
  unsafe {
    let ret = libc::syscall(libc::SYS_landlock_restrict_self, ruleset_fd as i64, 0);
    libc::close(ruleset_fd);
    if ret < 0 {
      return Err(format!("landlock_restrict_self failed: {}", std::io::Error::last_os_error()));
    }
  }
  Ok(())
}

// ─── seccomp-bpf ──────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn apply_seccomp_filter(level: SandboxLevel) -> Result<(), String> {
  use libc::{prctl, syscall, PR_SET_NO_NEW_PRIVS, SYS_seccomp};

  const SECCOMP_SET_MODE_FILTER: u8 = 2;
  const SECCOMP_FILTER_FLAG_TSYNC: u8 = 1;
  const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
  const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
  const EPERM: u32 = 1;

  let blacklist: &[i64] = match level {
    SandboxLevel::Basic => &BASIC_BLACKLIST,
    SandboxLevel::Strict => &STRICT_BLACKLIST,
    SandboxLevel::Maximum => &MAXIMUM_BLACKLIST,
    _ => return Ok(()),
  };

  unsafe {
    let ret = prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
    if ret != 0 {
      return Err(format!("prctl(PR_SET_NO_NEW_PRIVS) failed: {}", std::io::Error::last_os_error()));
    }
  }

  let filters = build_seccomp_bpf(blacklist, SECCOMP_RET_ALLOW, SECCOMP_RET_ERRNO | EPERM);

  unsafe {
    let prog = sock_fprog {
      len: filters.len() as u16,
      filter: filters.as_ptr(),
    };
    let ret = syscall(
      SYS_seccomp as i64,
      SECCOMP_SET_MODE_FILTER as i64,
      SECCOMP_FILTER_FLAG_TSYNC as i64,
      &prog as *const sock_fprog as i64,
    );
    if ret != 0 {
      return Err(format!("seccomp() failed: {}", std::io::Error::last_os_error()));
    }
  }

  Ok(())
}

#[repr(C)]
struct sock_filter {
  code: u16,
  jt: u8,
  jf: u8,
  k: u32,
}

#[repr(C)]
struct sock_fprog {
  len: u16,
  filter: *const sock_filter,
}

fn build_seccomp_bpf(blacklist: &[i64], default_action: u32, block_action: u32) -> Vec<sock_filter> {
  let mut insns = Vec::with_capacity(1 + blacklist.len() * 2 + 1);
  insns.push(sock_filter { code: 0x20, jt: 0, jf: 0, k: 0 });
  for &nr in blacklist {
    insns.push(sock_filter { code: 0x15, jt: 1, jf: 2, k: nr as u32 });
    insns.push(sock_filter { code: 0x06, jt: 0, jf: 0, k: block_action });
  }
  insns.push(sock_filter { code: 0x06, jt: 0, jf: 0, k: default_action });
  insns
}

const BASIC_BLACKLIST: &[i64] = &[
  56, 57, 58, 59, 61, 101, 246, 320, 321, 322, 435,
];

const STRICT_BLACKLIST: &[i64] = &[
  41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 53, 56, 57, 58, 59, 61, 101, 246,
  288, 299, 307, 320, 321, 322, 435,
];

const MAXIMUM_BLACKLIST: &[i64] = &[
  41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 53, 56, 57, 58, 59, 61,
  76, 77, 82, 83, 84, 85, 86, 87, 88, 90, 91, 92, 93, 94, 101,
  132, 133, 155, 161, 165, 166, 174, 175, 176, 235, 246,
  257, 258, 259, 260, 263, 264, 265, 266, 268, 272, 280, 285,
  288, 299, 307, 308, 313, 316, 320, 321, 322, 435,
];
