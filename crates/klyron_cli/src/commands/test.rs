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
    if args.ui {
        return TestRunner::run_ui(dir);
    }
    if args.e2e {
        return TestRunner::run_e2e(dir);
    }
    if args.unit {
        return TestRunner::run_unit(dir);
    }
    if args.integration {
        return TestRunner::run_integration(dir);
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
