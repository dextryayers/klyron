use clap::Args;
use klyron_linter::{Linter, LinterConfig};

#[derive(Args)]
pub struct LintArgs {
    pub dir: Option<std::path::PathBuf>,
    #[arg(long)]
    pub fix: bool,
    #[arg(long)]
    pub rules: Option<String>,
    #[arg(long, default_value = "stylish")]
    pub format: String,
    #[arg(long)]
    pub max_warnings: Option<u64>,
}

pub fn run_lint(args: LintArgs) -> anyhow::Result<()> {
    let dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap());
    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    let mut config = LinterConfig::default();
    config.max_warnings = args.max_warnings;

    let linter = Linter::with_config(config);
    let backend = linter.detect(&dir);

    let report = if args.fix {
        linter.lint_fix(&dir)?
    } else {
        linter.lint_dir(&dir, &[])?
    };

    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "junit" => {
            let test_suite = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="klyron lint" tests="{}" failures="{}" errors="{}">
  <properties>
    <property name="backend" value="{}"/>
  </properties>
  {}
</testsuite>"#,
                report.files_checked,
                report.total_errors,
                report.total_warnings,
                backend.name(),
                report.issues.iter().map(|issue| {
                    format!(
                        r#"  <testcase name="{}" classname="{}" line="{}" column="{}">
    <failure message="{}" type="{}"/>
  </testcase>"#,
                        issue.code, issue.file, issue.line, issue.column,
                        issue.message.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;"),
                        issue.level
                    )
                }).collect::<Vec<_>>().join("\n")
            );
            println!("{}", test_suite);
        }
        _ => {
            for issue in &report.issues {
                let color = if issue.level == "error" { "\x1b[31m" } else { "\x1b[33m" };
                println!(
                    "{}{}:{}:{}: {} ({})\x1b[0m",
                    color, issue.file, issue.line, issue.column, issue.message, issue.code
                );
            }
            println!(
                "Lint: {} errors, {} warnings in {} files (backend: {})",
                report.total_errors, report.total_warnings, report.files_checked, backend.name()
            );
        }
    }

    if report.total_errors > 0 {
        std::process::exit(1);
    }
    if let Some(max_w) = args.max_warnings {
        if report.total_warnings > max_w {
            eprintln!(
                "\x1b[31mMax warnings exceeded: {} > {}\x1b[0m",
                report.total_warnings, max_w
            );
            std::process::exit(1);
        }
    }

    Ok(())
}
