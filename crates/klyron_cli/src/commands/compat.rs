use clap::Args;
use klyron_compat::{CompatChecker, FrameworkTarget};

#[derive(Args)]
pub struct CompatArgs {
    pub target: Option<String>,
}

pub fn run_compat(args: CompatArgs) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match args.target.as_deref() {
        None | Some("check") => {
            let report = CompatChecker::check_project(&dir)?;
            println!("{}", report.summary);
            for check in &report.checks {
                println!("  {:?}: {} \u{2014} {}", check.status, check.name, check.message);
            }
            Ok(())
        }
        Some("react") => check_framework(&dir, FrameworkTarget::React),
        Some("next") => check_framework(&dir, FrameworkTarget::Next),
        Some("astro") => check_framework(&dir, FrameworkTarget::Astro),
        Some("nest") => check_framework(&dir, FrameworkTarget::Nest),
        Some("prisma") => check_framework(&dir, FrameworkTarget::Prisma),
        Some(t) => anyhow::bail!("Unknown compat target: {t}"),
    }
}

fn check_framework(dir: &std::path::Path, target: FrameworkTarget) -> anyhow::Result<()> {
    let report = CompatChecker::check_framework(dir, target)?;
    println!("{}", report.summary);
    for check in &report.checks {
        println!("  {:?}: {} \u{2014} {}", check.status, check.name, check.message);
    }
    Ok(())
}
