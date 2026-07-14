use clap::Args;

#[derive(Args)]
pub struct FormatArgs {
    pub dir: Option<std::path::PathBuf>,
    #[arg(long)]
    pub write: bool,
}

pub fn run_format(args: FormatArgs) -> anyhow::Result<()> {
    let dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }
    let project = crate::detect_project_type(&dir);

    if args.write {
        match project {
            "node" => crate::run_cmd("npx", &["prettier", "--write", "."], &dir),
            "laravel" => crate::run_cmd("php", &["vendor/bin/pint"], &dir),
            "rust" => crate::run_cmd("cargo", &["fmt"], &dir),
            "python" => crate::run_cmd("python3", &["-m", "black", "."], &dir),
            "ruby" => crate::run_cmd("bundle", &["exec", "rubocop", "-A"], &dir),
            "go" => crate::run_cmd("go", &["fmt", "./..."], &dir),
            _ => crate::run_cmd("npx", &["prettier", "--write", "."], &dir),
        }
    } else {
        match project {
            "node" => crate::run_cmd("npx", &["prettier", "--check", "."], &dir),
            "rust" => crate::run_cmd("cargo", &["fmt", "--check"], &dir),
            _ => crate::run_cmd("npx", &["prettier", "--check", "."], &dir),
        }
    }
}
