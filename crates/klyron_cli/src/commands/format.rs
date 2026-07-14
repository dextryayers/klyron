use clap::Args;
use klyron_formatter::Formatter;

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

    let report = if args.write {
        Formatter::format_write(&dir)?
    } else {
        Formatter::format_check(&dir)?
    };

    println!(
        "Format: {} changed, {} unchanged",
        report.files_changed, report.files_unchanged
    );
    Ok(())
}
