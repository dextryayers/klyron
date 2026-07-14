use clap::{Args, Subcommand};
use std::path::Path;
use std::process::Command;

#[derive(Args)]
pub struct DbArgs {
    #[arg(long = "orm", help = "ORM to use (prisma, drizzle, typeorm, mikroorm, sequelize, mongoose, kysely, knex)")]
    pub orm: Option<String>,
    #[command(subcommand)]
    pub action: DbAction,
}

#[derive(Subcommand)]
pub enum DbAction {
    /// Initialize ORM config
    Init,
    /// Generate schema/client
    Generate,
    /// Run migrations
    Migrate { #[arg(short, long)] name: Option<String> },
    /// Push schema to database
    Push,
    /// Pull schema from database
    Pull,
    /// Seed the database
    Seed { #[arg(short, long)] class: Option<String> },
    /// Reset/fresh migrate
    Reset,
    /// Open ORM studio
    Studio,
    /// Rollback last migration
    Rollback { #[arg(short, long)] step: Option<usize> },
    /// Format schema file
    Format,
    /// Validate schema
    Validate,
    /// Create migration
    Create { name: String },
}

pub fn run_db(action: DbAction, orm_override: Option<&str>) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    let orm = orm_override.map(|s| s.to_string()).unwrap_or_else(|| detect_orm(&dir));

    match action {
        DbAction::Init => orm_init(&orm, &dir),
        DbAction::Generate => orm_generate(&orm, &dir),
        DbAction::Migrate { name } => orm_migrate(&orm, &dir, name.as_deref()),
        DbAction::Push => orm_push(&orm, &dir),
        DbAction::Pull => orm_pull(&orm, &dir),
        DbAction::Seed { class } => orm_seed(&orm, &dir, class.as_deref()),
        DbAction::Reset => orm_reset(&orm, &dir),
        DbAction::Studio => orm_studio(&orm, &dir),
        DbAction::Rollback { step: _ } => orm_rollback(&orm, &dir),
        DbAction::Format => orm_format(&orm, &dir),
        DbAction::Validate => orm_validate(&orm, &dir),
        DbAction::Create { name } => orm_create(&orm, &dir, &name),
    }
}

// ── ORM Detection ──────────────────────────────────────────────────────────

pub fn detect_orm(dir: &Path) -> String {
    // Prisma
    if dir.join("schema.prisma").exists() || dir.join("prisma/schema.prisma").exists() {
        return "prisma".to_string();
    }
    // Drizzle
    if dir.join("drizzle.config.ts").exists() || dir.join("drizzle.config.js").exists() {
        return "drizzle".to_string();
    }
    // TypeORM
    if dir.join("ormconfig.json").exists() || dir.join("ormconfig.ts").exists() || dir.join("ormconfig.js").exists() {
        return "typeorm".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "typeorm") {
        return "typeorm".to_string();
    }
    // MikroORM
    if dir.join("mikro-orm.config.ts").exists() || dir.join("mikro-orm.config.js").exists() {
        return "mikroorm".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "@mikro-orm/core") {
        return "mikroorm".to_string();
    }
    // Sequelize
    if dir.join("config/config.json").exists() && has_npm_dep(dir, "sequelize").unwrap_or(false) {
        return "sequelize".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "sequelize") {
        if dir.join("models/index.js").exists() || dir.join("models").exists() {
            return "sequelize".to_string();
        }
    }
    // Mongoose
    if let Some(true) = has_npm_dep(dir, "mongoose") {
        return "mongoose".to_string();
    }
    // Kysely
    if dir.join("kysely.ts").exists() || dir.join("kysely.config.ts").exists() {
        return "kysely".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "kysely") {
        return "kysely".to_string();
    }
    // Knex
    if dir.join("knexfile.ts").exists() || dir.join("knexfile.js").exists() {
        return "knex".to_string();
    }
    if let Some(true) = has_npm_dep(dir, "knex") {
        return "knex".to_string();
    }
    "unknown".to_string()
}

fn has_npm_dep(dir: &Path, dep: &str) -> Option<bool> {
    let pkg = dir.join("package.json");
    let content = std::fs::read_to_string(pkg).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        if deps.contains_key(dep) { return Some(true); }
    }
    if let Some(deps) = json.get("devDependencies").and_then(|d| d.as_object()) {
        if deps.contains_key(dep) { return Some(true); }
    }
    Some(false)
}

// ── ORM-specific dispatch ──────────────────────────────────────────────────

fn orm_init(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "init"], dir),
        "drizzle" => run_npx(&["drizzle-kit", "init"], dir),
        "typeorm" => run_npx(&["typeorm", "init"], dir),
        "mikroorm" => run_npx(&["mikro-orm", "init"], dir),
        "sequelize" => run_npx(&["sequelize", "init"], dir),
        "mongoose" => {
            std::fs::create_dir_all(dir.join("models"))?;
            std::fs::write(dir.join("models/index.js"),
                r#"const mongoose = require('mongoose');
mongoose.connect(process.env.MONGO_URI || 'mongodb://localhost:27017/myapp');
module.exports = mongoose;
"#)?;
            println!("Mongoose setup complete. Add your schemas to models/");
            Ok(())
        }
        "kysely" => {
            std::fs::write(dir.join("kysely.ts"),
                r#"import { Kysely, PostgresDialect, Generated } from 'kysely';
import { Pool } from 'pg';
interface Database { person: PersonTable; }
interface PersonTable { id: Generated<number>; first_name: string; last_name: string; created_at: Date; }
const db = new Kysely<Database>({
  dialect: new PostgresDialect({ pool: new Pool({ connectionString: process.env.DATABASE_URL }) }),
});
export { db };
"#)?;
            println!("kysely.ts config created. Edit with your schema.");
            Ok(())
        }
        "knex" => run_npx(&["knex", "init"], dir),
        _ => {
            println!("No ORM detected. Initializing Prisma...");
            run_npx(&["prisma", "init"], dir)
        }
    }
}

fn orm_generate(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "generate"], dir),
        "drizzle" => run_npx(&["drizzle-kit", "generate"], dir),
        "typeorm" => run_npx(&["typeorm", "migration:generate", "-d", "src/data-source.ts"], dir),
        "mikroorm" => run_npx(&["mikro-orm", "migration:create"], dir),
        "sequelize" => run_npx(&["sequelize", "migration:generate", "--name", "init"], dir),
        "mongoose" => {
            println!("Mongoose: Create schemas manually in models/");
            Ok(())
        }
        "kysely" => run_npx(&["kysely", "generate"], dir),
        "knex" => run_npx(&["knex", "migrate:make", "init"], dir),
        _ => anyhow::bail!("No ORM detected. Run `klyron db init` first."),
    }
}

fn orm_migrate(orm: &str, dir: &Path, _name: Option<&str>) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "migrate", "dev"], dir),
        "drizzle" => run_npx(&["drizzle-kit", "migrate"], dir),
        "typeorm" => run_npx(&["typeorm", "migration:run"], dir),
        "mikroorm" => run_npx(&["mikro-orm", "migration:up"], dir),
        "sequelize" => run_npx(&["sequelize", "db:migrate"], dir),
        "mongoose" => {
            println!("Mongoose: Handle migration manually or use migrate-mongo.");
            Ok(())
        }
        "kysely" => run_npx(&["kysely", "migrate:latest"], dir),
        "knex" => run_npx(&["knex", "migrate:latest"], dir),
        _ => run_npx(&["prisma", "migrate", "dev"], dir),
    }
}

fn orm_push(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "db", "push"], dir),
        "drizzle" => run_npx(&["drizzle-kit", "push"], dir),
        "typeorm" => run_npx(&["typeorm", "schema:sync"], dir),
        "mikroorm" => run_npx(&["mikro-orm", "schema:update", "--run"], dir),
        "sequelize" => run_npx(&["sequelize", "db:sync"], dir),
        _ => anyhow::bail!("Push not supported for {orm}"),
    }
}

fn orm_pull(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "db", "pull"], dir),
        "drizzle" => run_npx(&["drizzle-kit", "pull"], dir),
        "typeorm" => run_npx(&["typeorm", "model:generate"], dir),
        "mikroorm" => run_npx(&["mikro-orm", "schema:dump"], dir),
        _ => anyhow::bail!("Pull not supported for {orm}"),
    }
}

fn orm_seed(orm: &str, dir: &Path, class: Option<&str>) -> anyhow::Result<()> {
    match orm {
        "prisma" => {
            if let Some(c) = class { run_npx(&["prisma", "db", "seed", "--", "--class", c], dir) }
            else { run_npx(&["prisma", "db", "seed"], dir) }
        }
        "mikroorm" => run_npx(&["mikro-orm", "seeder:run"], dir),
        "sequelize" => run_npx(&["sequelize", "db:seed:all"], dir),
        "knex" => run_npx(&["knex", "seed:run"], dir),
        _ => anyhow::bail!("Seed not supported for {orm}"),
    }
}

fn orm_reset(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "migrate", "reset", "--force"], dir),
        "typeorm" => {
            run_npx(&["typeorm", "schema:drop"], dir)?;
            run_npx(&["typeorm", "migration:run"], dir)
        }
        "mikroorm" => {
            run_npx(&["mikro-orm", "schema:drop", "--run"], dir)?;
            run_npx(&["mikro-orm", "migration:up"], dir)
        }
        "sequelize" => run_npx(&["sequelize", "db:migrate:undo:all"], dir),
        "knex" => run_npx(&["knex", "migrate:rollback", "--all"], dir),
        _ => anyhow::bail!("Reset not supported for {orm}"),
    }
}

fn orm_studio(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "studio"], dir),
        "drizzle" => run_npx(&["drizzle-kit", "studio"], dir),
        _ => anyhow::bail!("Studio not supported for {orm}"),
    }
}

fn orm_rollback(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "migrate", "dev", "--create-only"], dir),
        "typeorm" => run_npx(&["typeorm", "migration:revert"], dir),
        "mikroorm" => run_npx(&["mikro-orm", "migration:down"], dir),
        "sequelize" => run_npx(&["sequelize", "db:migrate:undo"], dir),
        "knex" => run_npx(&["knex", "migrate:rollback"], dir),
        _ => run_npx(&["prisma", "migrate", "dev", "--create-only"], dir),
    }
}

fn orm_format(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "format"], dir),
        _ => anyhow::bail!("Format not supported for {orm}"),
    }
}

fn orm_validate(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "validate"], dir),
        _ => anyhow::bail!("Validate not supported for {orm}"),
    }
}

fn orm_create(orm: &str, dir: &Path, name: &str) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "migrate", "dev", "--name", name], dir),
        "drizzle" => run_npx(&["drizzle-kit", "generate", "--name", name], dir),
        "typeorm" => run_npx(&["typeorm", "migration:create", name], dir),
        "mikroorm" => run_npx(&["mikro-orm", "migration:create", name], dir),
        "sequelize" => run_npx(&["sequelize", "migration:generate", "--name", name], dir),
        "knex" => run_npx(&["knex", "migrate:make", name], dir),
        _ => anyhow::bail!("Create migration not supported for {orm}"),
    }
}

// ── Helper ─────────────────────────────────────────────────────────────────

fn run_npx(args: &[&str], dir: &Path) -> anyhow::Result<()> {
    let status = Command::new("npx")
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run npx: {e}"))?;
    if !status.success() {
        anyhow::bail!("npx {} exited with code {}", args.join(" "), status);
    }
    Ok(())
}
