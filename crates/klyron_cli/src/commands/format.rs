use clap::Args;
use klyron_formatter::Formatter;
use std::io::Read;

#[derive(Args)]
pub struct FormatArgs {
    pub dir: Option<std::path::PathBuf>,
    #[arg(long)]
    pub write: bool,
    #[arg(long)]
    pub check: bool,
    #[arg(long)]
    pub stdin: bool,
}

pub fn run_format(args: FormatArgs) -> anyhow::Result<()> {
    if args.stdin {
        let dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());
        if !dir.exists() {
            anyhow::bail!("Directory not found: {}", dir.display());
        }
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        let formatter = Formatter::new();
        let report = formatter.format_stdin(&dir)?;
        print!("{}", report.output);
        return Ok(());
    }

    let dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    if args.check {
        let report = Formatter::format_check(&dir)?;
        let has_changes = report.files_changed > 0;

        if has_changes {
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

        println!(
            "Format check: {} would change, {} unchanged, {} skipped",
            report.files_changed, report.files_unchanged, report.files_skipped
        );

        if has_changes {
            std::process::exit(1);
        }
        return Ok(());
    }

    let report = if args.write {
        Formatter::format_write(&dir)?
    } else {
        Formatter::format_check(&dir)?
    };

    println!(
        "Format: {} changed, {} unchanged, {} skipped",
        report.files_changed, report.files_unchanged, report.files_skipped
    );

    let backend = klyron_formatter::Formatter::detect(&dir);
    println!("Backend: {}", backend.name());

    Ok(())
}
