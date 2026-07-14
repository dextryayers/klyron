use clap::Subcommand;

#[derive(Subcommand)]
pub enum PrismaAction {
    Generate,
    Migrate,
    Studio,
    DbPush,
}

#[derive(Subcommand)]
pub enum DrizzleAction {
    Generate,
    Migrate,
    Studio,
}

pub fn run_prisma(action: PrismaAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        PrismaAction::Generate => run_cmd_orm("npx", &["prisma", "generate"], &dir),
        PrismaAction::Migrate => run_cmd_orm("npx", &["prisma", "migrate", "dev"], &dir),
        PrismaAction::Studio => run_cmd_orm("npx", &["prisma", "studio"], &dir),
        PrismaAction::DbPush => run_cmd_orm("npx", &["prisma", "db", "push"], &dir),
    }
}

pub fn run_drizzle(action: DrizzleAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        DrizzleAction::Generate => run_cmd_orm("npx", &["drizzle-kit", "generate"], &dir),
        DrizzleAction::Migrate => run_cmd_orm("npx", &["drizzle-kit", "migrate"], &dir),
        DrizzleAction::Studio => run_cmd_orm("npx", &["drizzle-kit", "studio"], &dir),
    }
}

fn run_cmd_orm(program: &str, args: &[&str], dir: &std::path::Path) -> anyhow::Result<()> {
    use std::process::Command;
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run {}: {e}", program))?;
    if !status.success() {
        anyhow::bail!("{} exited with code {}", program, status);
    }
    Ok(())
}
