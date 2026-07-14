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
    pub ui: bool,
    #[arg(long)]
    pub e2e: bool,
    #[arg(long)]
    pub unit: bool,
    #[arg(long)]
    pub integration: bool,
}

pub fn run_test(args: TestArgs) -> anyhow::Result<()> {
    let dir = args.dir;
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }
    let project = crate::detect_project_type(&dir);

    if args.watch {
        println!("Running tests in watch mode...");
        return match project {
            "node" => crate::run_cmd("npx", &["vitest", "--watch"], &dir),
            _ => crate::run_cmd("npx", &["vitest", "--watch"], &dir),
        };
    }
    if args.coverage {
        println!("Running tests with coverage...");
        return match project {
            "node" => crate::run_cmd("npx", &["vitest", "--coverage"], &dir),
            "laravel" => crate::run_cmd("php", &["vendor/bin/phpunit", "--coverage-html", "coverage"], &dir),
            _ => crate::run_cmd("npx", &["vitest", "--coverage"], &dir),
        };
    }
    if args.ui {
        return crate::run_cmd("npx", &["vitest", "--ui"], &dir);
    }
    if args.e2e {
        return crate::run_cmd("npx", &["playwright", "test"], &dir);
    }
    if args.unit {
        return match project {
            "node" => crate::run_cmd("npx", &["vitest", "run", "--testPathPattern=unit"], &dir),
            _ => crate::run_cmd("npx", &["vitest", "run"], &dir),
        };
    }
    if args.integration {
        return match project {
            "node" => crate::run_cmd("npx", &["vitest", "run", "--testPathPattern=integration"], &dir),
            _ => crate::run_cmd("npx", &["vitest", "run"], &dir),
        };
    }

    match project {
        "node" => crate::run_cmd("npx", &["vitest", "run"], &dir),
        "laravel" => crate::run_cmd("php", &["vendor/bin/phpunit"], &dir),
        "python" => crate::run_cmd("python3", &["-m", "pytest"], &dir),
        "ruby" => crate::run_cmd("bundle", &["exec", "rspec"], &dir),
        "rust" => crate::run_cmd("cargo", &["test"], &dir),
        "go" => crate::run_cmd("go", &["test", "./..."], &dir),
        _ => {
            println!("No test runner configured for {project}, trying vitest...");
            crate::run_cmd("npx", &["vitest", "run"], &dir)
        }
    }
}
