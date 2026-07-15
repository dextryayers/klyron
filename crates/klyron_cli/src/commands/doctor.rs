use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct DoctorArgs {
    #[arg(long)]
    pub fix: bool,
}

pub fn run_doctor(fix: bool) -> anyhow::Result<()> {
    eprintln!(
        "{} Klyron System Diagnostic v{}",
        crate::Color::BOLD.paint("\u{1F3AF}"),
        env!("CARGO_PKG_VERSION"),
    );
    eprintln!(
        "{}",
        crate::Color::DIM.paint(format!(
            "Platform: {} | Arch: {}",
            std::env::consts::OS,
            std::env::consts::ARCH,
        ))
    );
    eprintln!();

    let mut all_ok = true;

    all_ok &= check_tool("git", "git --version", "Version control", fix);
    all_ok &= check_tool("node", "node --version", "JavaScript runtime", fix);
    all_ok &= check_tool("npm", "npm --version", "Node package manager", fix);
    all_ok &= check_tool("python3", "python3 --version", "Python 3 interpreter", fix);
    all_ok &= check_tool("rustc", "rustc --version", "Rust compiler", fix);
    all_ok &= check_tool("cargo", "cargo --version", "Rust package manager", fix);

    eprintln!();

    let cache_path = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("klyron");
    check_cache_dir(&cache_path, fix);
    check_home_writable(fix);
    check_disk_space(fix);
    check_path(fix);

    eprintln!();

    if all_ok {
        eprintln!(
            "{} All checks passed!",
            crate::Color::GREEN.paint("\u{2705}")
        );
    } else {
        eprintln!(
            "{} Some checks failed. Run with {} to attempt auto-repair.",
            crate::Color::YELLOW.paint("\u{26A0}"),
            crate::Color::BOLD.paint("--fix"),
        );
    }

    Ok(())
}

fn check_tool(
    name: &str,
    cmd_str: &str,
    description: &str,
    fix: bool,
) -> bool {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let program = parts[0];
    let args = &parts[1..];

    let output = std::process::Command::new(program)
        .args(args)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let ver = stdout
                .lines()
                .chain(stderr.lines())
                .next()
                .unwrap_or("")
                .trim();
            eprintln!(
                "  {} {:<8} {}",
                crate::Color::GREEN.paint("\u{2705}"),
                name,
                crate::Color::DIM.paint(ver),
            );
            true
        }
        _ => {
            eprintln!(
                "  {} {:<8} ({})",
                crate::Color::RED.paint("\u{274C}"),
                name,
                description,
            );
            if fix {
                match name {
                    "git" | "node" | "npm" | "python3" => {
                        eprintln!(
                            "    {} Install {} using your package manager (apt, brew, etc.)",
                            crate::Color::YELLOW.paint("\u{1F527}"),
                            name,
                        );
                    }
                    "rustc" | "cargo" => {
                        eprintln!(
                            "    {} Install Rust via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
                            crate::Color::YELLOW.paint("\u{1F527}"),
                        );
                    }
                    _ => {
                        eprintln!(
                            "    {} Install {} using your package manager",
                            crate::Color::YELLOW.paint("\u{1F527}"),
                            name,
                        );
                    }
                }
            }
            false
        }
    }
}

fn check_cache_dir(cache_path: &PathBuf, fix: bool) {
    let exists = cache_path.exists();
    let writable = exists && std::fs::metadata(cache_path)
        .map(|m| !m.permissions().readonly())
        .unwrap_or(false);

    if exists && writable {
        eprintln!(
            "  {} Cache directory: {}",
            crate::Color::GREEN.paint("\u{2705}"),
            cache_path.display(),
        );
    } else if !exists {
        eprintln!(
            "  {} Cache directory missing: {}",
            crate::Color::RED::paint("\u{274C}"),
            cache_path.display(),
        );
        if fix {
            match std::fs::create_dir_all(cache_path) {
                Ok(_) => eprintln!(
                    "    {} Created cache directory",
                    crate::Color::GREEN.paint("\u{1F527}"),
                ),
                Err(e) => eprintln!(
                    "    {} Could not create cache directory: {e}",
                    crate::Color::RED.paint("\u{1F527}"),
                ),
            }
        }
    } else {
        eprintln!(
            "  {} Cache directory not writable: {}",
            crate::Color::RED.paint("\u{274C}"),
            cache_path.display(),
        );
        if fix {
            match std::fs::set_permissions(cache_path, std::fs::Permissions::from_mode(0o755)) {
                Ok(_) => eprintln!(
                    "    {} Fixed permissions on cache directory",
                    crate::Color::GREEN.paint("\u{1F527}"),
                ),
                Err(e) => eprintln!(
                    "    {} Could not fix permissions: {e}",
                    crate::Color::RED.paint("\u{1F527}"),
                ),
            }
        }
    }
}

fn check_home_writable(fix: bool) {
    let klyron_home = std::env::var("KLYRON_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".klyron")
        });

    let ok = {
        std::fs::create_dir_all(&klyron_home).ok();
        std::fs::metadata(&klyron_home)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false)
    };

    if ok {
        eprintln!(
            "  {} KLYRON_HOME writable: {}",
            crate::Color::GREEN.paint("\u{2705}"),
            klyron_home.display(),
        );
    } else {
        eprintln!(
            "  {} KLYRON_HOME not writable: {}",
            crate::Color::RED.paint("\u{274C}"),
            klyron_home.display(),
        );
        if fix {
            match std::fs::set_permissions(&klyron_home, std::fs::Permissions::from_mode(0o755)) {
                Ok(_) => eprintln!(
                    "    {} Fixed permissions on KLYRON_HOME",
                    crate::Color::GREEN.paint("\u{1F527}"),
                ),
                Err(e) => eprintln!(
                    "    {} Could not fix: {e}",
                    crate::Color::RED.paint("\u{1F527}"),
                ),
            }
        }
    }
}

fn check_disk_space(fix: bool) {
    let klyron_home = std::env::var("KLYRON_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".klyron")
        });

    let free_gb = free_disk_space_gb(&klyron_home);

    if free_gb > 1.0 {
        eprintln!(
            "  {} Disk space: {:.1} GB free",
            crate::Color::GREEN.paint("\u{2705}"),
            free_gb,
        );
    } else {
        eprintln!(
            "  {} Low disk space: {:.1} GB free (need >1 GB)",
            crate::Color::RED.paint("\u{274C}"),
            free_gb,
        );
        if fix {
            eprintln!(
                "    {} Free up disk space by running: klyron cache clean --all",
                crate::Color::YELLOW.paint("\u{1F527}"),
            );
        }
    }
}

fn check_path(fix: bool) {
    let path_var = std::env::var("PATH").unwrap_or_default();
    let klyron_bin = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".klyron")
        .join("bin");
    let klyron_bin_str = klyron_bin.to_str().unwrap_or("");
    let in_path = path_var.split(':').any(|p| p == klyron_bin_str);

    if in_path {
        eprintln!(
            "  {} PATH includes ~/.klyron/bin",
            crate::Color::GREEN.paint("\u{2705}"),
        );
    } else {
        eprintln!(
            "  {} ~/.klyron/bin not in PATH",
            crate::Color::YELLOW.paint("\u{26A0}"),
        );
        if fix {
            let shell = std::env::var("SHELL").unwrap_or_default();
            let rc_file = if shell.ends_with("zsh") {
                "~/.zshrc"
            } else if shell.ends_with("bash") {
                "~/.bashrc"
            } else if shell.ends_with("fish") {
                "~/.config/fish/config.fish"
            } else {
                "~/.profile"
            };
            eprintln!(
                "    {} Add to {}: export PATH=\"$PATH:{}\"",
                crate::Color::YELLOW.paint("\u{1F527}"),
                rc_file,
                klyron_bin_str,
            );
            std::fs::create_dir_all(&klyron_bin).ok();
        }
    }
}

fn free_disk_space_gb(path: &PathBuf) -> f64 {
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        use std::ffi::CString;

        let cpath = CString::new(path.to_str().unwrap_or("/")).ok();
        unsafe {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            if let Some(ref p) = cpath {
                if libc::statvfs(p.as_ptr(), stat.as_mut_ptr()) == 0 {
                    let s = stat.assume_init();
                    let free_bytes = s.f_frsize as u64 * s.f_bavail as u64;
                    return free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                }
            }
        }
        0.0
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        100.0
    }
}
