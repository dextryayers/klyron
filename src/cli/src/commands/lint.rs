use clap::Args;

#[derive(Args)]
pub struct LintArgs {
    pub dir: Option<std::path::PathBuf>,
    #[arg(long)]
    pub fix: bool,
}

pub fn run_lint(args: LintArgs) -> anyhow::Result<()> {
    let dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }
    let project = crate::detect_project_type(&dir);

    if args.fix {
        match project {
            "node" => crate::run_cmd("npx", &["eslint", "--fix", "."], &dir),
            "laravel" => crate::run_cmd("php", &["vendor/bin/pint"], &dir),
            "rust" => crate::run_cmd("cargo", &["clippy", "--fix"], &dir),
            _ => crate::run_cmd("npx", &["eslint", "--fix", "."], &dir),
        }
    } else {
        match project {
            "node" => crate::run_cmd("npx", &["eslint", "."], &dir),
            "laravel" => crate::run_cmd("php", &["vendor/bin/pint", "--test"], &dir),
            "rust" => crate::run_cmd("cargo", &["clippy"], &dir),
            "python" => crate::run_cmd("python3", &["-m", "ruff", "check", "."], &dir),
            "ruby" => crate::run_cmd("bundle", &["exec", "rubocop"], &dir),
            "go" => crate::run_cmd("go", &["vet", "./..."], &dir),
            _ => crate::run_cmd("npx", &["eslint", "."], &dir),
        }
    }
}
