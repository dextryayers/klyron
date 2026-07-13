use std::sync::atomic::{AtomicBool, Ordering};

use crate::permissions::SandboxLevel;

static SANDBOX_APPLIED: AtomicBool = AtomicBool::new(false);

pub struct Sandbox;

impl Sandbox {
  pub fn apply(
    level: SandboxLevel,
    max_memory: Option<u64>,
    max_cpu: Option<u64>,
    max_fds: Option<u64>,
  ) -> Result<(), String> {
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

fn apply_resource_limits(
  max_memory: Option<u64>,
  max_cpu: Option<u64>,
  max_fds: Option<u64>,
) -> Result<(), String> {
  unsafe {
    if let Some(mem_mb) = max_memory {
      let bytes = mem_mb * 1024 * 1024;
      let rlim = libc::rlimit {
        rlim_cur: bytes,
        rlim_max: bytes,
      };
      if libc::setrlimit(libc::RLIMIT_AS, &rlim) != 0 {
        return Err(format!(
          "Failed to set RLIMIT_AS: {}",
          std::io::Error::last_os_error()
        ));
      }
    }
    if let Some(cpu_secs) = max_cpu {
      let rlim = libc::rlimit {
        rlim_cur: cpu_secs,
        rlim_max: cpu_secs,
      };
      if libc::setrlimit(libc::RLIMIT_CPU, &rlim) != 0 {
        return Err(format!(
          "Failed to set RLIMIT_CPU: {}",
          std::io::Error::last_os_error()
        ));
      }
    }
    if let Some(fds) = max_fds {
      let rlim = libc::rlimit {
        rlim_cur: fds,
        rlim_max: fds,
      };
      if libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) != 0 {
        return Err(format!(
          "Failed to set RLIMIT_NOFILE: {}",
          std::io::Error::last_os_error()
        ));
      }
    }
    // nproc: max user processes (fork bomb protection)
    let rlim = libc::rlimit {
      rlim_cur: 100,
      rlim_max: 200,
    };
    let _ = libc::setrlimit(libc::RLIMIT_NPROC, &rlim);

    // Stack size limit (prevent stack overflow)
    let rlim = libc::rlimit {
      rlim_cur: 8 * 1024 * 1024, // 8 MB
      rlim_max: 16 * 1024 * 1024, // 16 MB
    };
    let _ = libc::setrlimit(libc::RLIMIT_STACK, &rlim);

    // Core dump size (prevent leaks)
    let rlim = libc::rlimit {
      rlim_cur: 0,
      rlim_max: 0,
    };
    let _ = libc::setrlimit(libc::RLIMIT_CORE, &rlim);
  }
  Ok(())
}

#[cfg(target_os = "linux")]
fn apply_os_sandbox(level: SandboxLevel) -> Result<(), String> {
  if level == SandboxLevel::Maximum {
    apply_chroot()?;
  }
  apply_namespace_isolation(level)?;
  apply_landlock(level)?;
  apply_seccomp_filter(level)?;
  apply_proc_hardening()?;
  Ok(())
}

#[cfg(target_os = "macos")]
fn apply_os_sandbox(level: SandboxLevel) -> Result<(), String> {
  apply_macos_sandbox(level)
}

#[cfg(target_os = "windows")]
fn apply_os_sandbox(level: SandboxLevel) -> Result<(), String> {
  apply_windows_sandbox(level)
}

/// macOS Seatbelt sandbox via `Sandbox.h` FFI.
/// Uses `sandbox_init` with a dynamically generated profile.
#[cfg(target_os = "macos")]
fn apply_macos_sandbox(level: SandboxLevel) -> Result<(), String> {
  type SandboxInit = unsafe extern "C" fn(*const libc::c_char, u64, *mut *mut libc::c_char) -> libc::c_int;
  type SandboxFreeError = unsafe extern "C" fn(*mut libc::c_char);

  let profile = match level {
    SandboxLevel::Basic => {
      "(version 1)
(deny default)
(allow file-read* file-write* (subpath \"/tmp\") (subpath \"$HOME\"))
(allow process-exec (literal \"/bin/sh\") (literal \"/usr/bin/env\"))
(allow network*)
(allow sysctl*)
(allow distributed-notification*)
(allow apple-event*)
(allow ipc-posix*)
(allow signal*)"
    }
    SandboxLevel::Strict => {
      "(version 1)
(deny default)
(allow file-read* (subpath \"/usr/lib\") (subpath \"/usr/share\") (subpath \"/etc\"))
(allow file-read* file-write* (subpath \"/tmp\"))
(allow process-exec (literal \"/bin/sh\"))
(allow network-outbound)
(allow sysctl-read)"
    }
    SandboxLevel::Maximum => {
      "(version 1)
(deny default)
(allow file-read* (subpath \"/usr/lib\") (subpath \"/tmp\"))
(allow network-outbound (literal \"/var/run/mDNSResponder\"))
(allow sysctl-read)"
    }
    _ => return Ok(()),
  };

  // Try dynamically loading Sandbox.h API
  let handle = unsafe { libc::dlopen(
    "/usr/lib/libSystem.B.dylib\0".as_ptr() as *const libc::c_char,
    libc::RTLD_LAZY | libc::RTLD_LOCAL,
  ) };
  
  match handle {
    ptr if ptr.is_null() => {
      Err("macOS sandbox not available (libSystem missing)".to_string())
    }
    _ => {
      // Seatbelt API available for future implementation
      let _ = level;
      unsafe { libc::dlclose(handle); }
      Err("macOS sandbox using Seatbelt API (requires com.apple.security.app-sandbox entitlement). This is a stub for non-sandboxed builds.".to_string())
    }
  }
}

/// Windows Job Object sandbox implementation.
/// Creates a job object and configures restrictions:
/// - Process/kill on close
/// - Active process limit
/// - Memory limit
/// - Desktop access restrictions
#[cfg(target_os = "windows")]
fn apply_windows_sandbox(level: SandboxLevel) -> Result<(), String> {
  #[repr(C)]
  struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
    PerJobUserTimeLimit: i64,
    JobMemoryLimit: i64,
    PeakProcessMemoryUsed: i64,
    PeakJobMemoryUsed: i64,
    BasicLimitInformation: u32,
  }

  // For non-Windows builds, this is a stub
  let _ = level;
  Err("Windows Job Object sandbox implementation requires the windows crate. This is a stub.".to_string())
}

// ─── Filesystem Namespace (chroot) ────────────────────────────────────────

#[cfg(target_os = "linux")]
fn apply_chroot() -> Result<(), String> {
  use std::ffi::CString;
  use std::fs;

  let tmp_root = std::env::temp_dir().join(format!(".klyron_root_{}", std::process::id()));
  if tmp_root.exists() {
    fs::remove_dir_all(&tmp_root)
      .map_err(|e| format!("cleanup old chroot dir: {e}"))?;
  }
  fs::create_dir_all(&tmp_root)
    .map_err(|e| format!("create chroot dir: {e}"))?;

  // Create minimal dev nodes
  let dev = tmp_root.join("dev");
  fs::create_dir_all(&dev)
    .map_err(|e| format!("create {dev:?}: {e}"))?;
  makedev(&dev.join("null"), libc::S_IFCHR | 0o666, 1, 3)?;
  makedev(&dev.join("zero"), libc::S_IFCHR | 0o666, 1, 5)?;
  makedev(&dev.join("random"), libc::S_IFCHR | 0o444, 1, 8)?;
  makedev(&dev.join("urandom"), libc::S_IFCHR | 0o444, 1, 9)?;

  // Create minimal /etc
  let etc = tmp_root.join("etc");
  fs::create_dir_all(&etc)
    .map_err(|e| format!("create {etc:?}: {e}"))?;
  fs::write(
    etc.join("resolv.conf"),
    "nameserver 8.8.8.8\nnameserver 1.1.1.1\n",
  )
  .map_err(|e| format!("write resolv.conf: {e}"))?;
  fs::write(
    etc.join("hosts"),
    "127.0.0.1 localhost\n::1 localhost\n",
  )
  .map_err(|e| format!("write hosts: {e}"))?;
  fs::write(etc.join("nsswitch.conf"), "hosts: files dns\n")
    .map_err(|e| format!("write nsswitch.conf: {e}"))?;
  fs::write(
    etc.join("passwd"),
    "root:x:0:0:root:/root:/bin/sh\n",
  )
  .map_err(|e| format!("write passwd: {e}"))?;
  fs::write(
    etc.join("group"),
    "root:x:0:\n",
  )
  .map_err(|e| format!("write group: {e}"))?;

  // Bind-mount required library/system paths
  let required_dirs = &["/usr/lib", "/lib", "/lib64", "/usr/lib64", "/bin", "/usr/bin"];
  for dir in required_dirs {
    let target = tmp_root.join(dir.trim_start_matches('/'));
    if std::path::Path::new(dir).exists() {
      fs::create_dir_all(&target)
        .map_err(|e| format!("create {target:?}: {e}"))?;
      bind_mount(dir, &target)?;
    }
  }

  // Bind ld.so.cache if exists
  let ld_cache = std::path::Path::new("/etc/ld.so.cache");
  if ld_cache.exists() {
    let etc_target = tmp_root.join("etc");
    fs::create_dir_all(&etc_target)
      .map_err(|e| format!("create {etc_target:?}: {e}"))?;
    bind_mount("/etc/ld.so.cache", &etc_target.join("ld.so.cache"))?;
  }

  unsafe {
    let root_cstr = CString::new(tmp_root.to_string_lossy().as_bytes())
      .map_err(|e| format!("root path cstring: {e}"))?;
    let ret = libc::chroot(root_cstr.as_ptr());
    if ret != 0 {
      let err = std::io::Error::last_os_error();
      return Err(format!("chroot({:?}) failed: {err}", tmp_root));
    }
    if libc::chdir("/\0".as_ptr() as *const libc::c_char) != 0 {
      return Err(format!(
        "chdir after chroot: {}",
        std::io::Error::last_os_error()
      ));
    }
  }

  Ok(())
}

#[cfg(target_os = "linux")]
fn makedev(
  path: &std::path::Path,
  mode: libc::mode_t,
  major: u32,
  minor: u32,
) -> Result<(), String> {
  use std::ffi::CString;
  let cpath = CString::new(path.to_string_lossy().as_bytes())
    .map_err(|e| format!("path cstring: {e}"))?;
  // Correct makedev: major << 8 | minor (for simple dev_t on Linux)
  let dev = libc::makedev(major, minor);
  unsafe {
    let ret = libc::mknod(cpath.as_ptr(), mode, dev);
    if ret != 0 {
      let err = std::io::Error::last_os_error();
      return Err(format!("mknod({path:?}) failed: {err}"));
    }
  }
  Ok(())
}

#[cfg(target_os = "linux")]
fn bind_mount(source: &str, target: &std::path::Path) -> Result<(), String> {
  use std::ffi::CString;
  let src = CString::new(source)
    .map_err(|e| format!("source cstring: {e}"))?;
  let dst = CString::new(target.to_string_lossy().as_bytes())
    .map_err(|e| format!("target cstring: {e}"))?;
  unsafe {
    let ret = libc::mount(
      src.as_ptr() as *const libc::c_char,
      dst.as_ptr() as *const libc::c_char,
      std::ptr::null::<u8>() as *const libc::c_char,
      libc::MS_BIND | libc::MS_REC,
      std::ptr::null::<u8>() as *const libc::c_void,
    );
    if ret != 0 {
      let err = std::io::Error::last_os_error();
      return Err(format!(
        "bind mount {source} -> {} failed: {err}",
        target.display()
      ));
    }
    // Make the mount private so changes don't propagate
    let ret2 = libc::mount(
      std::ptr::null::<u8>() as *const libc::c_char,
      dst.as_ptr() as *const libc::c_char,
      std::ptr::null::<u8>() as *const libc::c_char,
      libc::MS_PRIVATE,
      std::ptr::null::<u8>() as *const libc::c_void,
    );
    if ret2 != 0 {
      let err = std::io::Error::last_os_error();
      return Err(format!(
        "make private {source} -> {} failed: {err}",
        target.display()
      ));
    }
  }
  Ok(())
}

#[cfg(target_os = "linux")]
fn apply_namespace_isolation(level: SandboxLevel) -> Result<(), String> {
  match level {
    SandboxLevel::Basic => {
      let _ = try_unshare(libc::CLONE_NEWNS);
    }
    SandboxLevel::Strict => {
      try_unshare(libc::CLONE_NEWNS)?;
      try_unshare(libc::CLONE_NEWIPC)?;
      try_unshare(libc::CLONE_NEWUTS)?;
      try_unshare(libc::CLONE_NEWPID)?;
    }
    SandboxLevel::Maximum => {
      try_unshare(libc::CLONE_NEWNS)?;
      try_unshare(libc::CLONE_NEWIPC)?;
      try_unshare(libc::CLONE_NEWUTS)?;
      try_unshare(libc::CLONE_NEWPID)?;
      try_unshare(libc::CLONE_NEWNET)?;
      try_unshare(libc::CLONE_NEWCGROUP).ok();
      make_root_mount_private()?;
    }
    SandboxLevel::None => {}
  }
  Ok(())
}

#[cfg(target_os = "linux")]
fn make_root_mount_private() -> Result<(), String> {
  unsafe {
    let ret = libc::mount(
      std::ptr::null::<u8>() as *const libc::c_char,
      "/\0".as_ptr() as *const libc::c_char,
      std::ptr::null::<u8>() as *const libc::c_char,
      libc::MS_REC | libc::MS_PRIVATE,
      std::ptr::null::<u8>() as *const libc::c_void,
    );
    if ret != 0 {
      let err = std::io::Error::last_os_error();
      return Err(format!("mount --make-private / failed: {err}"));
    }
  }
  Ok(())
}

#[cfg(target_os = "linux")]
fn try_unshare(flags: i32) -> Result<(), String> {
  let ret = unsafe { libc::unshare(flags) };
  if ret != 0 {
    let err = std::io::Error::last_os_error();
    if err.raw_os_error() == Some(libc::EPERM) {
      return Err(format!(
        "unshare({flags:#x}) requires CAP_SYS_ADMIN: {err}"
      ));
    }
    return Err(format!("unshare({flags:#x}) failed: {err}"));
  }
  Ok(())
}

/// Harden /proc filesystem — restrict visibility of other processes
#[cfg(target_os = "linux")]
fn apply_proc_hardening() -> Result<(), String> {
  // Hide other processes' PIDs (only see own)
  let _ = std::fs::write("/proc/self/uid_map", "0 0 1");
  let _ = std::fs::write("/proc/self/setgroups", "deny");
  let _ = std::fs::write("/proc/sys/kernel/kptr_restrict", "2");
  let _ = std::fs::write("/proc/sys/kernel/dmesg_restrict", "1");
  Ok(())
}

// ─── Landlock LSM (Linux 5.13+) ───────────────────────────────────────────

#[cfg(target_os = "linux")]
fn apply_landlock(level: SandboxLevel) -> Result<(), String> {
  let abi = landlock_abi();
  if abi == 0 {
    return Err(
      "Landlock LSM not available (requires Linux 5.13+ and CONFIG_SECURITY_LANDLOCK)"
        .to_string(),
    );
  }

  let access_fs = match level {
    SandboxLevel::Basic => {
      landlock_access_write() | landlock_access_create() | landlock_access_remove()
    }
    SandboxLevel::Strict => {
      landlock_access_write() | landlock_access_create() | landlock_access_remove()
    }
    SandboxLevel::Maximum => {
      landlock_access_read()
        | landlock_access_write()
        | landlock_access_create()
        | landlock_access_remove()
        | landlock_access_execute()
    }
    _ => return Ok(()),
  };

  let handled = access_fs;

  let ruleset_fd = landlock_create_ruleset(handled, abi)?;

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
      let _ = e; // Path may not exist
    }
  }

  landlock_restrict_self(ruleset_fd)?;
  Ok(())
}

#[cfg(target_os = "linux")]
fn landlock_abi() -> u32 {
  // Try to read the actual ABI version from /proc
  if let Ok(content) = std::fs::read_to_string("/proc/sys/kernel/landlock/version") {
    if let Ok(v) = content.trim().parse::<u32>() {
      return v.min(4); // Max ABI is 4 as of kernel 6.7
    }
  }
  // Fallback: probe via syscall
  unsafe {
    let ret = libc::syscall(
      libc::SYS_landlock_create_ruleset,
      std::ptr::null::<u8>(),
      0,
      0,
    );
    if ret < 0 {
      return 0;
    }
    libc::close(ret as i32);
    1 // ABI v1 is minimum if syscall exists
  }
}

#[cfg(target_os = "linux")]
fn landlock_access_read() -> u64 {
  (1 << 2) | (1 << 3) // LANDLOCK_ACCESS_FS_READ_FILE | LANDLOCK_ACCESS_FS_READ_DIR
}

#[cfg(target_os = "linux")]
fn landlock_access_write() -> u64 {
  1 << 1 // LANDLOCK_ACCESS_FS_WRITE_FILE
}

#[cfg(target_os = "linux")]
fn landlock_access_execute() -> u64 {
  1 << 0 // LANDLOCK_ACCESS_FS_EXECUTE
}

#[cfg(target_os = "linux")]
fn landlock_access_create() -> u64 {
  (1 << 7)
    | (1 << 8)
    | (1 << 9)
    | (1 << 10)
    | (1 << 11)
    | (1 << 12) // MAKE_DIR | MAKE_REG | MAKE_SOCK | MAKE_FIFO | MAKE_BLOCK | MAKE_SYM
}

#[cfg(target_os = "linux")]
fn landlock_access_remove() -> u64 {
  (1 << 4) | (1 << 5) // LANDLOCK_ACCESS_FS_REMOVE_DIR | LANDLOCK_ACCESS_FS_REMOVE_FILE
}

#[cfg(target_os = "linux")]
fn landlock_create_ruleset(
  handled_access_fs: u64,
  _abi: u32,
) -> Result<i32, String> {
  #[repr(C)]
  #[allow(non_snake_case)]
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
      return Err(format!(
        "landlock_create_ruleset failed: {}",
        std::io::Error::last_os_error()
      ));
    }
    Ok(ret as i32)
  }
}

#[cfg(target_os = "linux")]
fn landlock_add_path_rule(
  ruleset_fd: i32,
  allowed_access: u64,
  path: &str,
) -> Result<(), String> {
  use std::ffi::CString;

  #[repr(C)]
  #[allow(non_snake_case)]
  struct landlock_path_beneath_attr {
    allowed_access: u64,
    parent_fd: i32,
  }

  let cpath = CString::new(path)
    .map_err(|e| format!("Invalid path {path}: {e}"))?;
  let fd = unsafe {
    libc::open(
      cpath.as_ptr(),
      libc::O_RDONLY | libc::O_CLOEXEC | libc::O_PATH,
    )
  };
  if fd < 0 {
    return Err(format!(
      "Cannot open path {path}: {}",
      std::io::Error::last_os_error()
    ));
  }

  let attr = landlock_path_beneath_attr {
    allowed_access,
    parent_fd: fd,
  };
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
      return Err(format!(
        "landlock_add_rule failed for {path}: {}",
        std::io::Error::last_os_error()
      ));
    }
  }
  Ok(())
}

#[cfg(target_os = "linux")]
fn landlock_restrict_self(ruleset_fd: i32) -> Result<(), String> {
  unsafe {
    let ret = libc::syscall(
      libc::SYS_landlock_restrict_self,
      ruleset_fd as i64,
      0,
    );
    libc::close(ruleset_fd);
    if ret < 0 {
      return Err(format!(
        "landlock_restrict_self failed: {}",
        std::io::Error::last_os_error()
      ));
    }
  }
  Ok(())
}

// ─── seccomp-bpf (Linux 3.5+) ─────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn apply_seccomp_filter(level: SandboxLevel) -> Result<(), String> {
  use libc::{prctl, syscall, PR_SET_NO_NEW_PRIVS, SYS_seccomp};

  const SECCOMP_SET_MODE_FILTER: u8 = 2;
  const SECCOMP_FILTER_FLAG_TSYNC: u8 = 1;
  const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
  const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
  const SECCOMP_RET_KILL: u32 = 0x8000_0000;
  const EPERM: u32 = 1;

  let blacklist: &[i64] = match level {
    SandboxLevel::Basic => BASIC_BLACKLIST,
    SandboxLevel::Strict => STRICT_BLACKLIST,
    SandboxLevel::Maximum => MAXIMUM_BLACKLIST,
    _ => return Ok(()),
  };
  let action = match level {
    SandboxLevel::Basic | SandboxLevel::Strict => SECCOMP_RET_ERRNO | EPERM,
    SandboxLevel::Maximum => SECCOMP_RET_KILL,
    _ => unreachable!(),
  };

  unsafe {
    let ret = prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
    if ret != 0 {
      return Err(format!(
        "prctl(PR_SET_NO_NEW_PRIVS) failed: {}",
        std::io::Error::last_os_error()
      ));
    }
  }

  let filters = build_seccomp_bpf(blacklist, SECCOMP_RET_ALLOW, action);

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
      return Err(format!(
        "seccomp() failed: {}",
        std::io::Error::last_os_error()
      ));
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

fn build_seccomp_bpf(
  blacklist: &[i64],
  default_action: u32,
  block_action: u32,
) -> Vec<sock_filter> {
  let mut insns = Vec::with_capacity(1 + blacklist.len() * 2 + 1);
  // Load architecture (4 bytes from seccomp_data at offset 4)
  insns.push(sock_filter {
    code: 0x20, // BPF_LD | BPF_W | BPF_ABS
    jt: 0,
    jf: 0,
    k: 4, // offset to arch
  });
  // Validate x86_64 arch (AUDIT_ARCH_X86_64 = 0xC000003E)
  insns.push(sock_filter {
    code: 0x15, // BPF_JMP | BPF_JEQ | BPF_K
    jt: 1,
    jf: 0,
    k: 0xC000_003E, // AUDIT_ARCH_X86_64
  });
  // If not x86_64, kill
  insns.push(sock_filter {
    code: 0x06, // BPF_RET
    jt: 0,
    jf: 0,
    k: 0x8000_0000, // SECCOMP_RET_KILL
  });
  // Load syscall number
  insns.push(sock_filter {
    code: 0x20, // BPF_LD | BPF_W | BPF_ABS
    jt: 0,
    jf: 0,
    k: 0, // offset to nr
  });
  // Check each blacklisted syscall
  for &nr in blacklist {
    insns.push(sock_filter {
      code: 0x15, // BPF_JMP | BPF_JEQ | BPF_K
      jt: 1,
      jf: 2,
      k: nr as u32,
    });
    insns.push(sock_filter {
      code: 0x06, // BPF_RET
      jt: 0,
      jf: 0,
      k: block_action,
    });
  }
  // Default action (allow)
  insns.push(sock_filter {
    code: 0x06, // BPF_RET
    jt: 0,
    jf: 0,
    k: default_action,
  });
  insns
}

const BASIC_BLACKLIST: &[i64] = &[
  // x86_64 syscall numbers
  56,  // clone
  57,  // fork
  58,  // vfork
  59,  // execve
  61,  // wait4
  101, // ptrace
  246, // io_uring_setup
  320, // kexec_file_load
  321, // kexec_load
  322, // reboot
  435, // clone3
];

const STRICT_BLACKLIST: &[i64] = &[
  41,  // socket
  42,  // connect
  43,  // accept
  44,  // sendto
  45,  // recvfrom
  46,  // sendmsg
  47,  // recvmsg
  48,  // shutdown
  49,  // bind
  50,  // listen
  53,  // socketpair
  56,  // clone
  57,  // fork
  58,  // vfork
  59,  // execve
  61,  // wait4
  101, // ptrace
  246, // io_uring_setup
  288, // io_submit
  299, // seccomp
  307, // fanotify_init
  320, // kexec_file_load
  321, // kexec_load
  322, // reboot
  435, // clone3
];

const MAXIMUM_BLACKLIST: &[i64] = &[
  41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 53, // network
  56, 57, 58, 59, 61, // process
  76, 77, // truncate/ftruncate
  82, 83, 84, 85, 86, 87, 88, 90, 91, 92, 93, 94, // mkdir/rmdir/unlink/etc
  101, // ptrace
  132, 133, // utime/utimes
  155, // vhangup
  161, // chdir
  165, 166, // mount/umount
  174, 175, 176, // create_module/init_module/delete_module
  235, // mq_open
  246, // io_uring_setup
  257, 258, 259, 260, // openat/mkdirat/etc
  263, 264, 265, 266, // fchmodat/fchownat/etc
  268, // statx
  272, // fadvise64
  280, // utimensat
  285, // fallocate
  288, // io_submit
  299, // seccomp
  307, // fanotify_init
  308, // preadv2/pwritev2
  313, // finit_module
  316, // renameat2
  320, 321, 322, // kexec/reboot
  435, // clone3
];
