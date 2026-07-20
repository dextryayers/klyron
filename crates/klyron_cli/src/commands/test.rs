use clap::Args;

#[derive(Args)]
pub struct TestArgs {
    #[arg(default_value = ".")]
    pub dir: std::path::PathBuf,
    #[arg(long)]
    pub filter: Option<String>,
    #[arg(long)]
    pub watch: bool,
    #[arg(long)]
    pub coverage: bool,
    #[arg(long)]
    pub verbose: bool,
    #[arg(last = true)]
    pub args: Vec<String>,
}

pub fn run_test(args: TestArgs) -> anyhow::Result<()> {
    let dir = &args.dir;
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    crate::load_dotenv(dir);
    let project = crate::detect_project_type(dir);

    let runner = detect_test_runner(dir);
    if runner != "unknown" {
        return run_external_runner(dir, runner, &args);
    }

    if project == "rust" {
        return crate::run_cmd("cargo", &["test"], dir);
    }
    if project == "go" {
        return crate::run_cmd("go", &["test", "./..."], dir);
    }

    let pm = crate::detect_package_runner(dir);
    let mut pm_args = vec!["test".to_string()];
    if !args.args.is_empty() {
        pm_args.push("--".to_string());
        pm_args.extend(args.args.iter().cloned());
    }
    let status = std::process::Command::new(pm)
        .args(&pm_args)
        .current_dir(dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run {} test: {e}", pm))?;
    if !status.success() {
        anyhow::bail!("{} test failed", pm);
    }
    Ok(())
}

fn run_external_runner(dir: &std::path::Path, runner: &str, args: &TestArgs) -> anyhow::Result<()> {
    let (cmd, base_args) = match runner {
        "vitest" => ("npx", vec!["vitest", "run"]),
        "jest" => ("npx", vec!["jest"]),
        "mocha" => ("npx", vec!["mocha"]),
        "phpunit" => ("php", vec!["vendor/bin/phpunit"]),
        "pest" => ("php", vec!["vendor/bin/pest"]),
        "pytest" => ("python3", vec!["-m", "pytest"]),
        "rspec" => ("bundle", vec!["exec", "rspec"]),
        _ => return crate::run_cmd("npm", &["test"], dir),
    };

    let mut full_args: Vec<String> = base_args.iter().map(|s| s.to_string()).collect();
    if args.watch {
        full_args.push("--watch".to_string());
    }
    if args.coverage {
        full_args.push("--coverage".to_string());
    }
    if !args.args.is_empty() {
        full_args.push("--".to_string());
        full_args.extend(args.args.iter().cloned());
    }

    let status = std::process::Command::new(cmd)
        .args(&full_args)
        .current_dir(dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run {}: {e}", runner))?;
    if !status.success() {
        anyhow::bail!("{} exited with code {}", runner, status.code().unwrap_or(-1));
    }
    Ok(())
}

fn detect_test_runner(dir: &std::path::Path) -> &'static str {
    if dir.join("vitest.config.ts").exists() || dir.join("vitest.config.js").exists() {
        "vitest"
    } else if dir.join("jest.config.ts").exists() || dir.join("jest.config.js").exists() {
        "jest"
    } else if has_npm_dep(dir, "vitest") {
        "vitest"
    } else if has_npm_dep(dir, "jest") {
        "jest"
    } else if has_npm_dep(dir, "mocha") {
        "mocha"
    } else if dir.join("phpunit.xml").exists() || dir.join("phpunit.xml.dist").exists() {
        "phpunit"
    } else if dir.join("pest.xml").exists() || dir.join("pest").exists() {
        "pest"
    } else if dir.join("pytest.ini").exists() || dir.join("pyproject.toml").exists() {
        "pytest"
    } else if dir.join("Gemfile").exists() {
        "rspec"
    } else if dir.join("Cargo.toml").exists() {
        "cargo"
    } else if dir.join("go.mod").exists() {
        "go"
    } else {
        "unknown"
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
