use clap::Args;
use klyron_test::{TestRunner, TestRunnerConfig};

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
    pub shuffle: bool,
    #[arg(long)]
    pub verbose: bool,
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

    crate::load_dotenv(dir);

    if let Some(tsconfig) = crate::detect_tsconfig(dir) {
        let _opts = crate::apply_tsconfig_compiler_options(&tsconfig);
    }

    let project_type = crate::detect_project_type(dir);
    let external_runner = detect_test_runner(dir);
    println!("Detected project type: {project_type}, test runner: {external_runner}");

    // Use native JS runner when no external tool is detected and JS/TS test files exist
    if external_runner == "unknown" && (project_type == "node" || project_type == "typescript") {
        let test_files = klyron_test::discover_js_test_files(dir);
        if !test_files.is_empty() && !klyron_test::has_test_framework_dep(dir) {
            println!("Using klyron native JS test runner ({} files)", test_files.len());
            return run_native_js_tests(dir, &test_files, &args);
        }
    }

    if args.shuffle {
        eprintln!("Shuffle mode: randomized test order");
    }

    if args.verbose {
        eprintln!("Verbose output enabled");
    }

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

    let config = TestRunnerConfig {
        parallel: !args.shuffle,
        ..Default::default()
    };
    let _runner = TestRunner::with_config(config);

    if args.ui || args.e2e || args.unit || args.integration {
        let category = if args.ui { "ui" } else if args.e2e { "e2e" } else if args.unit { "unit" } else { "integration" };
        println!("Running {category} tests...");
        let result = TestRunner::run(dir, args.filter.as_deref())?;
        if args.verbose {
            println!("--- Test Output ---");
            println!("{}", result.output);
            println!("--- End Output ---");
        }
        println!(
            "\x1b[{}mTests: {} passed, {} failed, {} skipped in {:.2}s\x1b[0m",
            if result.failed > 0 { "31" } else { "32" },
            result.passed, result.failed, result.skipped, result.time
        );
        if result.failed > 0 {
            std::process::exit(1);
        }
        return Ok(());
    }

    let result = TestRunner::run(dir, args.filter.as_deref())?;
    if args.verbose {
        println!("--- Test Output ---");
        println!("{}", result.output);
        println!("--- End Output ---");
    }
    println!(
        "\x1b[{}mTests: {} passed, {} failed, {} skipped in {:.2}s\x1b[0m",
        if result.failed > 0 { "31" } else { "32" },
        result.passed, result.failed, result.skipped, result.time
    );
    if result.failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn run_native_js_tests(
    dir: &std::path::Path,
    test_files: &[std::path::PathBuf],
    args: &TestArgs,
) -> anyhow::Result<()> {
    use klyron_core::Runtime;
    use klyron_test::assertions::prepare_js_test;
    use std::time::Instant;

    let start = Instant::now();
    let mut total_passed = 0u64;
    let mut total_failed = 0u64;

    let runtime = Runtime::builder()
        .enable_typescript(test_files.iter().any(|f| {
            f.extension().map(|e| e == "ts" || e == "tsx").unwrap_or(false)
        }))
        .build()?;

    for file_path in test_files {
        let file_name = file_path
            .strip_prefix(dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        let code = std::fs::read_to_string(file_path)?;
        let wrapped = prepare_js_test(&code);

        match runtime.eval(&wrapped) {
            Ok(output) => {
                // Parse the JSON output from the last line
                let parsed = output
                    .lines()
                    .filter_map(|l| {
                        let trimmed = l.trim();
                        if trimmed.starts_with("{\"") || trimmed.starts_with("{\"__klyron_suites") {
                            serde_json::from_str::<serde_json::Value>(trimmed).ok()
                        } else {
                            None
                        }
                    })
                    .last();

                if let Some(json) = parsed {
                    if let Some(suites) = json.get("__klyron_suites").and_then(|v| v.as_array()) {
                        for suite in suites {
                            let suite_name = suite
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unnamed");
                            if let Some(tests) = suite.get("tests").and_then(|v| v.as_array()) {
                                for test_case in tests {
                                    let test_name = test_case
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unnamed");

                                    let failed_assertions = test_case
                                        .get("assertions")
                                        .and_then(|v| v.as_array())
                                        .map(|a| {
                                            a.iter()
                                                .filter(|a| {
                                                    a.get("passed")
                                                        .and_then(|v| v.as_bool())
                                                        .unwrap_or(false)
                                                        == false
                                                })
                                                .count()
                                        })
                                        .unwrap_or(0);

                                    if failed_assertions > 0 {
                                        total_failed += 1;
                                        if args.verbose {
                                            println!("  FAIL  {suite_name} > {test_name}");
                                            for a in test_case
                                                .get("assertions")
                                                .and_then(|v| v.as_array())
                                                .unwrap_or(&vec![])
                                            {
                                                if a.get("passed")
                                                    .and_then(|v| v.as_bool())
                                                    .unwrap_or(false)
                                                    == false
                                                {
                                                    if let Some(err) =
                                                        a.get("error").and_then(|v| v.as_str())
                                                    {
                                                        println!("    {err}");
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        total_passed += 1;
                                        if args.verbose {
                                            println!("  PASS  {suite_name} > {test_name}");
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if args.verbose {
                    println!("  {file_name}: {output}");
                }
            }
            Err(e) => {
                total_failed += 1;
                eprintln!("  FAIL  {file_name}: {e}");
            }
        }
    }

    let elapsed = start.elapsed();
    let color = if total_failed > 0 { "31" } else { "32" };
    println!(
        "\x1b[{color}mTests: {total_passed} passed, {total_failed} failed, {} total in {:.2}s\x1b[0m",
        total_passed + total_failed,
        elapsed.as_secs_f64(),
    );
    if total_failed > 0 {
        std::process::exit(1);
    }
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
