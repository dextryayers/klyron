use clap::Args;
use klyron_test::TestRunner;

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
    pub ui: bool,
    #[arg(long)]
    pub e2e: bool,
    #[arg(long)]
    pub unit: bool,
    #[arg(long)]
    pub integration: bool,
    #[arg(long)]
    pub backend: Option<String>,
}

pub fn run_test(args: TestArgs) -> anyhow::Result<()> {
    let dir = &args.dir;
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    let project_type = crate::detect_project_type(dir);
    let runner = detect_test_runner(dir);
    println!("Detected project type: {project_type}, test runner: {runner}");

    if let Some(backend) = &args.backend {
        println!("Forcing test backend: {backend}");
    }

    if args.watch {
        println!("Running tests in watch mode...");
        return TestRunner::run_watch(dir);
    }
    if args.coverage {
        println!("Running tests with coverage...");
        return TestRunner::run_coverage(dir);
    }
    if args.ui || args.e2e || args.unit || args.integration {
        let category = if args.ui { "ui" } else if args.e2e { "e2e" } else if args.unit { "unit" } else { "integration" };
        println!("Running {category} tests...");
        let result = TestRunner::run(dir, args.filter.as_deref())?;
        println!(
            "Tests: {} passed, {} failed, {} skipped in {:.2}s",
            result.passed, result.failed, result.skipped, result.time
        );
        return Ok(());
    }

    let result = TestRunner::run(dir, args.filter.as_deref())?;
    println!(
        "Tests: {} passed, {} failed, {} skipped in {:.2}s",
        result.passed, result.failed, result.skipped, result.time
    );
    Ok(())
}

fn detect_test_runner(dir: &std::path::Path) -> &'static str {
    if dir.join("phpunit.xml").exists() || dir.join("phpunit.xml.dist").exists() {
        "phpunit"
    } else if dir.join("pest.xml").exists() || dir.join("pest").exists() {
        "pest"
    } else if dir.join("vitest.config.ts").exists() || dir.join("vitest.config.js").exists() {
        "vitest"
    } else if dir.join("jest.config.ts").exists() || dir.join("jest.config.js").exists() {
        "jest"
    } else if has_npm_dep_check(dir, "mocha") {
        "mocha"
    } else if has_npm_dep_check(dir, "ava") {
        "ava"
    } else if has_npm_dep_check(dir, "tape") {
        "tape"
    } else if dir.join("Cargo.toml").exists() {
        "cargo test"
    } else if dir.join("go.mod").exists() {
        "go test"
    } else if dir.join("Gemfile").exists() {
        "rspec"
    } else if dir.join("pytest.ini").exists() || dir.join("pyproject.toml").exists() {
        "pytest"
    } else {
        "unknown"
    }
}

fn has_npm_dep_check(dir: &std::path::Path, dep: &str) -> bool {
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
