use clap::Subcommand;
use std::process::Command;

#[derive(Subcommand)]
pub enum DbAction {
    Init,
    Generate,
    Migrate { #[arg(short, long)] step: Option<usize> },
    Push,
    Pull,
    Seed { #[arg(short, long)] class: Option<String> },
    Reset,
    Studio,
    Rollback { #[arg(short, long)] step: Option<usize> },
}

pub fn run_db(action: DbAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let orm = detect_orm(&dir);

    match action {
        DbAction::Init => match orm.as_str() {
            "prisma" => run_cmd_gen("npx", &["prisma", "init"], &dir),
            "drizzle" => run_cmd_gen("npx", &["drizzle-kit", "init"], &dir),
            _ => {
                println!("No ORM detected. Initializing Prisma...");
                run_cmd_gen("npx", &["prisma", "init"], &dir)
            }
        },
        DbAction::Generate => match orm.as_str() {
            "prisma" => run_cmd_gen("npx", &["prisma", "generate"], &dir),
            "drizzle" => run_cmd_gen("npx", &["drizzle-kit", "generate"], &dir),
            _ => anyhow::bail!("No ORM detected. Run `klyron db init` first."),
        },
        DbAction::Migrate { step: _ } => match orm.as_str() {
            "prisma" => run_cmd_gen("npx", &["prisma", "migrate", "dev"], &dir),
            "drizzle" => run_cmd_gen("npx", &["drizzle-kit", "migrate"], &dir),
            _ => run_cmd_gen("npx", &["prisma", "migrate", "dev"], &dir),
        },
        DbAction::Push => match orm.as_str() {
            "prisma" => run_cmd_gen("npx", &["prisma", "db", "push"], &dir),
            "drizzle" => run_cmd_gen("npx", &["drizzle-kit", "push"], &dir),
            _ => anyhow::bail!("Push not supported for {orm}"),
        },
        DbAction::Pull => match orm.as_str() {
            "prisma" => run_cmd_gen("npx", &["prisma", "db", "pull"], &dir),
            "drizzle" => run_cmd_gen("npx", &["drizzle-kit", "pull"], &dir),
            _ => anyhow::bail!("Pull not supported for {orm}"),
        },
        DbAction::Seed { class } => {
            if let Some(c) = class {
                run_cmd_gen("npx", &["prisma", "db", "seed", "--", "--class", &c], &dir)
            } else {
                run_cmd_gen("npx", &["prisma", "db", "seed"], &dir)
            }
        }
        DbAction::Reset => match orm.as_str() {
            "prisma" => run_cmd_gen("npx", &["prisma", "migrate", "reset", "--force"], &dir),
            _ => anyhow::bail!("Reset not supported for {orm}"),
        },
        DbAction::Studio => match orm.as_str() {
            "prisma" => run_cmd_gen("npx", &["prisma", "studio"], &dir),
            "drizzle" => run_cmd_gen("npx", &["drizzle-kit", "studio"], &dir),
            _ => anyhow::bail!("Studio not supported for {orm}"),
        },
        DbAction::Rollback { step: _ } => match orm.as_str() {
            "prisma" => run_cmd_gen("npx", &["prisma", "migrate", "dev", "--create-only"], &dir),
            _ => run_cmd_gen("npx", &["prisma", "migrate", "dev", "--create-only"], &dir),
        },
    }
}

fn detect_orm(dir: &std::path::Path) -> String {
    if dir.join("schema.prisma").exists() { return "prisma".to_string(); }
    if dir.join("drizzle.config.ts").exists() || dir.join("drizzle.config.js").exists() { return "drizzle".to_string(); }
    if dir.join("ormconfig.json").exists() { return "typeorm".to_string(); }
    if dir.join("knexfile.ts").exists() || dir.join("knexfile.js").exists() { return "knex".to_string(); }
    "unknown".to_string()
}

fn run_cmd_gen(program: &str, args: &[&str], dir: &std::path::Path) -> anyhow::Result<()> {
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
