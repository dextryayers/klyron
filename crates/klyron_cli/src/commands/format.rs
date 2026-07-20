use clap::Args;
use klyron_formatter::Formatter;

#[derive(Args)]
pub struct FormatArgs {
    pub dir: Option<std::path::PathBuf>,
    #[arg(long)]
    pub write: bool,
    #[arg(long)]
    pub check: bool,
}

pub fn run_format(args: FormatArgs) -> anyhow::Result<()> {
    let dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    // Delegate to native formatter if available
    if let Some((cmd, subcmd)) = detect_native_formatter(&dir) {
        let mut fwd_args: Vec<&str> = subcmd.split_whitespace().collect();
        if args.check {
            fwd_args.push("--check");
        }
        let status = std::process::Command::new(cmd)
            .args(&fwd_args)
            .current_dir(&dir)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to run formatter: {e}"))?;
        if !status.success() {
            anyhow::bail!("Format check failed");
        }
        return Ok(());
    }

    // Fallback: try npm run format
    let pm = crate::detect_package_runner(&dir);
    let mut pm_args = vec!["run".to_string(), "format".to_string()];
    if args.check {
        pm_args.push("--".to_string());
        pm_args.push("--check".to_string());
    }
    let status = std::process::Command::new(pm)
        .args(&pm_args)
        .current_dir(&dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run format: {e}"))?;
    if status.success() {
        return Ok(());
    }

    // Last resort: use klyron's native formatter
    let report = if args.write {
        Formatter::format_write(&dir)?
    } else {
        let report = Formatter::format_check(&dir)?;
        if report.files_changed > 0 {
            for diff in &report.diffs {
                println!("Would format: {}", diff.file);
                for change in &diff.changes {
                    match change.tag.as_str() {
                        "Insert" => println!("  \x1b[32m+ {}\x1b[0m", change.content.trim_end()),
                        "Delete" => println!("  \x1b[31m- {}\x1b[0m", change.content.trim_end()),
                        _ => println!("    {}", change.content.trim_end()),
                    }
                }
            }
        }
        report
    };

    println!(
        "Format: {} changed, {} unchanged, {} skipped",
        report.files_changed, report.files_unchanged, report.files_skipped
    );

    Ok(())
}

fn detect_native_formatter(dir: &std::path::Path) -> Option<(&'static str, &'static str)> {
    if has_npm_dep(dir, "prettier") {
        Some(("npx", "prettier --write ."))
    } else if has_npm_dep(dir, "biome") {
        Some(("npx", "biome format --write"))
    } else if has_npm_dep(dir, "dprint") {
        Some(("npx", "dprint fmt"))
    } else if dir.join("Cargo.toml").exists() {
        Some(("cargo", "fmt"))
    } else if dir.join("go.mod").exists() {
        Some(("go", "fmt ./..."))
    } else {
        None
    }
}

fn has_npm_dep(dir: &std::path::Path, dep: &str) -> bool {
    let pkg = dir.join("package.json");
    let content = match std::fs::read_to_string(pkg) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        if deps.contains_key(dep) { return true; }
    }
    if let Some(deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
        if deps.contains_key(dep) { return true; }
    }
    false
}
