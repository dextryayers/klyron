use clap::Args;
use klyron_linter::Linter;

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

    let report = if args.fix {
        Linter::new().lint_fix(&dir)?
    } else {
        Linter::lint(&dir)?
    };

    println!(
        "Lint: {} errors, {} warnings in {} files",
        report.total_errors, report.total_warnings, report.files_checked
    );
    Ok(())
}
