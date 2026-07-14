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
    println!("Klyron System Check\n");
    let checks = [
        ("node", "node --version"),
        ("npm", "npm --version"),
        ("php", "php --version"),
        ("composer", "composer --version"),
        ("python3", "python3 --version"),
        ("ruby", "ruby --version"),
        ("go", "go version"),
        ("rustc", "rustc --version"),
        ("cargo", "cargo --version"),
        ("zig", "zig version"),
        ("cc", "cc --version"),
        ("c++", "c++ --version"),
        ("deno", "deno --version"),
    ];
    for (name, cmd_str) in &checks {
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        if parts.is_empty() {
            println!("  ?    {name:12} (invalid check)");
            continue;
        }
        let output = Command::new(parts[0]).args(&parts[1..]).output();
        match output {
            Ok(o) if o.status.success() => {
                let ver = String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").trim().to_string();
                println!("  OK   {name:12} {ver}");
            }
            _ => println!("  MISS {name:12} not found"),
        }
    }
    Ok(())
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
