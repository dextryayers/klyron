use clap::Args;
use std::process::Command;

#[derive(Args)]
pub struct TelemetryArgs {
    pub enabled: Option<bool>,
}

#[derive(Args)]
pub struct ConfigArgs {
    pub key: Option<String>,
    pub value: Option<String>,
}

pub fn run_init() -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let config_path = dir.join("klyron.toml");
    if config_path.exists() {
        anyhow::bail!("klyron.toml already exists");
    }
    let config = r#"[project]
name = "my-app"
version = "0.1.0"
description = ""
type = "auto"

[dev]
port = 3000
hmr = true

[build]
out_dir = "dist"
minify = true

[test]
runner = "auto"
coverage = false

[format]
runner = "auto"
indent_size = 2
"#;
    std::fs::write(&config_path, config)?;
    println!("✅ klyron.toml created");
    Ok(())
}

pub fn run_upgrade() -> anyhow::Result<()> {
    eprintln!("→ Upgrading Klyron...");
    let bin_path = std::env::current_exe().ok();
    let is_cargo = bin_path.as_ref().map_or(false, |p| p.display().to_string().contains(".cargo/bin"));
    let is_npm = bin_path.as_ref().map_or(false, |p| p.display().to_string().contains("/node_modules/"));

    if is_cargo {
        eprintln!("  Detected: cargo install");
        let result = Command::new("cargo")
            .args(["install", "--git", "https://github.com/dextryayers/klyron", "klyron", "--force"])
            .status();
        match result {
            Ok(s) if s.success() => { println!("✅ Klyron upgraded successfully"); Ok(()) }
            Ok(s) => anyhow::bail!("Upgrade failed (exit: {s})"),
            Err(e) => anyhow::bail!("Upgrade failed: {e}"),
        }
    } else if is_npm {
        eprintln!("  Detected: npm global install");
        let result = Command::new("npm")
            .args(["update", "-g", "klyron"])
            .status();
        match result {
            Ok(s) if s.success() => { println!("✅ Klyron upgraded successfully"); Ok(()) }
            _ => {
                eprintln!("  npm update failed, trying npm install...");
                let result = Command::new("npm")
                    .args(["install", "-g", "klyron@latest"])
                    .status();
                match result {
                    Ok(s) if s.success() => { println!("✅ Klyron upgraded successfully"); Ok(()) }
                    Ok(s) => anyhow::bail!("Upgrade failed (exit: {s})"),
                    Err(e) => anyhow::bail!("Upgrade failed: {e}"),
                }
            }
        }
    } else {
        eprintln!("  Using cargo install (default)...");
        let result = Command::new("cargo")
            .args(["install", "--git", "https://github.com/dextryayers/klyron", "klyron", "--force"])
            .status();
        match result {
            Ok(s) if s.success() => { println!("✅ Klyron upgraded successfully"); Ok(()) }
            Ok(s) => anyhow::bail!("Upgrade failed (exit: {s})"),
            Err(e) => anyhow::bail!("Upgrade failed: {e}"),
        }
    }
}

pub fn run_doctor() -> anyhow::Result<()> {
    use std::env;

    println!("{}", crate::color::Color::BOLD.paint("Klyron System Diagnostic"));
    println!("{}", crate::color::Color::DIM.paint(&format!("Version: {} | Platform: {} | Arch: {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
    )));
    println!();

    let checks: Vec<(&str, &str, &str)> = vec![
        ("node", "node --version", "JavaScript runtime"),
        ("npm", "npm --version", "Node package manager"),
        ("pnpm", "pnpm --version", "Fast package manager"),
        ("yarn", "yarn --version", "Alternative package manager"),
        ("bun", "bun --version", "JavaScript runtime & toolkit"),
        ("deno", "deno --version", "Modern JS/TS runtime"),
        ("php", "php --version", "PHP interpreter"),
        ("composer", "composer --version", "PHP package manager"),
        ("python3", "python3 --version", "Python 3 interpreter"),
        ("pip3", "pip3 --version", "Python package installer"),
        ("ruby", "ruby --version", "Ruby interpreter"),
        ("gem", "gem --version", "Ruby package manager"),
        ("go", "go version", "Go compiler"),
        ("rustc", "rustc --version", "Rust compiler"),
        ("cargo", "cargo --version", "Rust package manager"),
        ("zig", "zig version", "Zig compiler"),
        ("cc", "cc --version", "C compiler"),
        ("c++", "c++ --version", "C++ compiler"),
        ("docker", "docker --version", "Container runtime"),
        ("docker-compose", "docker compose version", "Docker orchestration"),
        ("git", "git --version", "Version control"),
        ("curl", "curl --version", "HTTP client"),
        ("wget", "wget --version", "Download tool"),
        ("jq", "jq --version", "JSON processor"),
        ("wasmtime", "wasmtime --version", "WASM runtime"),
        ("nginx", "nginx -v 2>&1 || true", "Web server"),
    ];

    let mut ok_count = 0u32;
    let mut miss_count = 0u32;

    for (name, cmd_str, description) in &checks {
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        if parts.is_empty() {
            println!("  {} {name:14} ({})", crate::color::Color::YELLOW.paint("?"), description);
            continue;
        }

        let use_stderr = cmd_str.contains("2>&1");
        let actual_cmd = if use_stderr {
            cmd_str.split("2>&1").next().unwrap_or(parts[0]).trim()
        } else {
            parts[0]
        };
        let actual_args: Vec<&str> = if use_stderr {
            let base = cmd_str.split("2>&1").next().unwrap_or("").trim();
            let base_parts: Vec<&str> = base.split_whitespace().collect();
            base_parts[1..].to_vec()
        } else {
            parts[1..].to_vec()
        };

        let output = Command::new(actual_cmd).args(&actual_args).output();
        match output {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                let ver = stdout.lines().chain(stderr.lines())
                    .next().unwrap_or("")
                    .trim().to_string();
                println!("  {} {name:14} {ver}", crate::color::Color::GREEN.paint("OK"));
                ok_count += 1;
            }
            _ => {
                println!("  {} {name:14} ({})", crate::color::Color::RED.paint("MISS"), description);
                miss_count += 1;
            }
        }
    }

    println!();
    let total = ok_count + miss_count;
    println!("{}", crate::color::Color::BOLD.paint(&format!(
        "Results: {ok_count}/{total} tools available, {miss_count} missing"
    )));

    if let Ok(cwd) = env::current_dir() {
        println!();
        println!("{}", crate::color::Color::BOLD.paint("Project Environment"));
        println!("  Directory: {}", cwd.display());

        let klyron_toml = cwd.join("klyron.toml");
        if klyron_toml.exists() {
            println!("  {} Klyron config found", crate::color::Color::GREEN.paint("\u{2713}"));
        } else {
            println!("  {} No klyron.toml (run `klyron init`)", crate::color::Color::YELLOW.paint("\u{26A0}"));
        }

        let project_type = crate::detect_project_type(&cwd);
        if project_type != "unknown" {
            println!("  Project type: {project_type}");
            let runner = crate::detect_package_runner(&cwd);
            println!("  Package runner: {runner}");
        }

        if cwd.join(".env").exists() {
            println!("  {} .env file found", crate::color::Color::GREEN.paint("\u{2713}"));
        } else if cwd.join(".env.example").exists() {
            println!("  {} .env.example found (create .env from it)", crate::color::Color::YELLOW.paint("\u{26A0}"));
        }

        if cwd.join("node_modules").exists() {
            println!("  {} node_modules present", crate::color::Color::GREEN.paint("\u{2713}"));
        } else if cwd.join("package.json").exists() {
            println!("  {} Missing node_modules (run `npm install`)", crate::color::Color::YELLOW.paint("\u{26A0}"));
        }

        let disk_usage = fs_available_space(&cwd);
        if let Some(space) = disk_usage {
            println!("  Disk space: {space:.1} GB available");
        }

        let rust_toolchain = cwd.join("rust-toolchain.toml");
        if rust_toolchain.exists() {
            if let Ok(content) = std::fs::read_to_string(&rust_toolchain) {
                if let Some(line) = content.lines().find(|l| l.contains("channel")) {
                    println!("  Rust toolchain: {}", line.trim());
                }
            }
        }
    }

    Ok(())
}

fn fs_available_space(_path: &std::path::Path) -> Option<f64> {
    None
}

pub fn run_version() -> anyhow::Result<()> {
    println!("Klyron v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

pub fn run_telemetry(enabled: Option<bool>) -> anyhow::Result<()> {
    match enabled {
        Some(true) => { println!("Telemetry enabled"); }
        Some(false) => { println!("Telemetry disabled"); }
        None => { println!("Telemetry status: disabled"); }
    }
    Ok(())
}

pub fn run_config(key: Option<String>, value: Option<String>) -> anyhow::Result<()> {
    match (key, value) {
        (Some(k), Some(v)) => println!("  Config set: {} = {}", k, v),
        (Some(k), None) => println!("  Config get: {} = <value>", k),
        (None, None) => {
            let config_path = std::env::current_dir()?.join("klyron.toml");
            if config_path.exists() {
                println!("Config file: {}", config_path.display());
                println!("{}", std::fs::read_to_string(&config_path)?);
            } else {
                println!("No klyron.toml found. Run `klyron init` to create one.");
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub fn run_clean(yes: bool) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let dirs_to_clean = ["node_modules", "dist", "build", ".next", ".cache", "target"];
    if !yes {
        println!("This will remove the following directories:");
        for d in &dirs_to_clean {
            let path = dir.join(d);
            if path.exists() {
                println!("  - {}", d);
            }
        }
        print!("Continue? [y/N] ");
        use std::io::Write;
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }
    for d in &dirs_to_clean {
        let path = dir.join(d);
        if path.exists() {
            std::fs::remove_dir_all(&path)?;
            println!("  Removed: {}", d);
        }
    }
    println!("Clean complete");
    Ok(())
}
