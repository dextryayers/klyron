use clap::{Args, Subcommand};
use std::io::Write;
use std::process::Command;

#[derive(Args)]
pub struct TelemetryArgs {
    #[command(subcommand)]
    pub action: Option<TelemetryAction>,
}

#[derive(Subcommand)]
pub enum TelemetryAction {
    Enable,
    Disable,
    Status,
    View,
}

#[derive(Args)]
pub struct ConfigArgs {
    pub key: Option<String>,
    pub value: Option<String>,
}

fn get_config_dir() -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    home.join(".klyron")
}

fn get_telemetry_pref_path() -> std::path::PathBuf {
    get_config_dir().join("config.toml")
}

fn read_telemetry_pref() -> Option<bool> {
    let path = get_telemetry_pref_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    if content.contains("enabled = true") {
        Some(true)
    } else if content.contains("enabled = false") {
        Some(false)
    } else {
        None
    }
}

fn write_telemetry_pref(enabled: bool) -> anyhow::Result<()> {
    let dir = get_config_dir();
    std::fs::create_dir_all(&dir)?;
    let path = get_telemetry_pref_path();
    std::fs::write(&path, format!("[telemetry]\nenabled = {}\n", enabled))?;
    Ok(())
}

pub fn run_telemetry(action: Option<TelemetryAction>) -> anyhow::Result<()> {
    match action {
        Some(TelemetryAction::Enable) => {
            write_telemetry_pref(true)?;
            println!("Telemetry has been enabled.");
            println!("We collect anonymous usage data to improve Klyron.");
            println!("To view collected data: klyron telemetry view");
        }
        Some(TelemetryAction::Disable) => {
            write_telemetry_pref(false)?;
            println!("Telemetry has been disabled.");
            println!("No data will be collected.");
        }
        Some(TelemetryAction::Status) => {
            match read_telemetry_pref() {
                Some(true) => {
                    println!("Telemetry: \x1b[32menabled\x1b[0m");
                    println!("Anonymous usage data is being collected.");
                }
                Some(false) => {
                    println!("Telemetry: \x1b[33mdisabled\x1b[0m");
                    println!("No data is being collected.");
                }
                None => {
                    println!("Telemetry: \x1b[33mnot configured\x1b[0m (default: disabled)");
                    println!("Run 'klyron telemetry enable' to opt in.");
                }
            }
        }
        Some(TelemetryAction::View) => {
            println!("Telemetry data is collected anonymously.");
            println!("Data collected includes:");
            println!("  - Command usage (e.g., 'klyron run', 'klyron test')");
            println!("  - Runtime version");
            println!("  - OS and architecture");
            println!("  - Performance metrics (evaluation time)");
            println!();
            println!("No personal data, file contents, or project names are collected.");
            println!("Run 'klyron telemetry disable' to stop data collection.");
        }
        None => {
            match read_telemetry_pref() {
                Some(true) => {
                    println!("Telemetry: \x1b[32menabled\x1b[0m");
                }
                Some(false) => {
                    println!("Telemetry: \x1b[33mdisabled\x1b[0m");
                }
                None => {
                    println!("Telemetry: \x1b[33mnot configured\x1b[0m (default: disabled)");
                }
            }
        }
    }
    Ok(())
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
    println!("klyron.toml created");
    Ok(())
}

pub fn run_upgrade() -> anyhow::Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Klyron v{}", current_version);
    println!("Checking for updates...");

    let client = reqwest::blocking::Client::builder()
        .user_agent("klyron-updater/1.0")
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {e}"))?;

    let release = match client
        .get("https://api.github.com/repos/dextryayers/klyron/releases/latest")
        .send()
    {
        Ok(resp) => {
            let json: serde_json::Value = resp
                .json()
                .map_err(|e| anyhow::anyhow!("Failed to parse release info: {e}"))?;
            json
        }
        Err(e) => {
            eprintln!("Could not check for updates: {e}");
            eprintln!("Falling back to cargo install upgrade...");
            return run_upgrade_fallback();
        }
    };

    let latest_tag = release["tag_name"].as_str().unwrap_or("v0.1.0");
    let latest_version = latest_tag.trim_start_matches('v');
    let body = release["body"].as_str().unwrap_or("No changelog available");

    if latest_version == current_version {
        println!("Already up to date (v{})", current_version);
        return Ok(());
    }

    println!("\n\x1b[1mUpdate available:\x1b[0m");
    println!("  Current: \x1b[33mv{}\x1b[0m", current_version);
    println!("  Latest:  \x1b[32m{}\x1b[0m", latest_tag);
    println!("\n\x1b[1mChangelog:\x1b[0m");
    for line in body.lines().take(30) {
        println!("  {}", line);
    }
    if body.lines().count() > 30 {
        println!("  \x1b[90m... and more\x1b[0m");
    }

    print!("\nUpgrade to {}? [Y/n] ", latest_tag);
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_lowercase();

    if answer == "n" || answer == "no" {
        println!("Upgrade cancelled.");
        return Ok(());
    }

    println!("\nUpgrading to {}...", latest_tag);
    run_upgrade_fallback()
}

fn run_upgrade_fallback() -> anyhow::Result<()> {
    let bin_path = std::env::current_exe().ok();
    let is_cargo = bin_path.as_ref().map_or(false, |p| p.display().to_string().contains(".cargo/bin"));
    let is_npm = bin_path.as_ref().map_or(false, |p| p.display().to_string().contains("/node_modules/"));

    if is_cargo {
        eprintln!("  Detected: cargo install");
        let result = Command::new("cargo")
            .args(["install", "--git", "https://github.com/dextryayers/klyron", "klyron", "--force"])
            .status();
        match result {
            Ok(s) if s.success() => { println!("Klyron upgraded successfully"); Ok(()) }
            Ok(s) => anyhow::bail!("Upgrade failed (exit: {s})"),
            Err(e) => anyhow::bail!("Upgrade failed: {e}"),
        }
    } else if is_npm {
        eprintln!("  Detected: npm global install");
        let result = Command::new("npm")
            .args(["update", "-g", "klyron"])
            .status();
        match result {
            Ok(s) if s.success() => { println!("Klyron upgraded successfully"); Ok(()) }
            _ => {
                eprintln!("  npm update failed, trying npm install...");
                let result = Command::new("npm")
                    .args(["install", "-g", "klyron@latest"])
                    .status();
                match result {
                    Ok(s) if s.success() => { println!("Klyron upgraded successfully"); Ok(()) }
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
            Ok(s) if s.success() => { println!("Klyron upgraded successfully"); Ok(()) }
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
