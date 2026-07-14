use crate::commands::helpers;
use clap::{Args, Subcommand};
use std::path::Path;

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
    let orm = orm_override.map(|s| s.to_string()).unwrap_or_else(|| helpers::detect_orm(&dir));

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

// ── ORM-specific dispatch ──────────────────────────────────────────────────

fn orm_init(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "init"], dir),
        "drizzle" => helpers::run_npx(&["drizzle-kit", "init"], dir),
        "typeorm" => helpers::run_npx(&["typeorm", "init"], dir),
        "mikroorm" => helpers::run_npx(&["mikro-orm", "init"], dir),
        "sequelize" => helpers::run_npx(&["sequelize", "init"], dir),
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
        "knex" => helpers::run_npx(&["knex", "init"], dir),
        _ => {
            println!("No ORM detected. Initializing Prisma...");
            helpers::run_npx(&["prisma", "init"], dir)
        }
    }
}

fn orm_generate(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "generate"], dir),
        "drizzle" => helpers::run_npx(&["drizzle-kit", "generate"], dir),
        "typeorm" => helpers::run_npx(&["typeorm", "migration:generate", "-d", "src/data-source.ts"], dir),
        "mikroorm" => helpers::run_npx(&["mikro-orm", "migration:create"], dir),
        "sequelize" => helpers::run_npx(&["sequelize", "migration:generate", "--name", "init"], dir),
        "mongoose" => {
            println!("Mongoose: Create schemas manually in models/");
            Ok(())
        }
        "kysely" => helpers::run_npx(&["kysely", "generate"], dir),
        "knex" => helpers::run_npx(&["knex", "migrate:make", "init"], dir),
        _ => anyhow::bail!("No ORM detected. Run `klyron db init` first."),
    }
}

fn orm_migrate(orm: &str, dir: &Path, _name: Option<&str>) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "migrate", "dev"], dir),
        "drizzle" => helpers::run_npx(&["drizzle-kit", "migrate"], dir),
        "typeorm" => helpers::run_npx(&["typeorm", "migration:run"], dir),
        "mikroorm" => helpers::run_npx(&["mikro-orm", "migration:up"], dir),
        "sequelize" => helpers::run_npx(&["sequelize", "db:migrate"], dir),
        "mongoose" => {
            println!("Mongoose: Handle migration manually or use migrate-mongo.");
            Ok(())
        }
        "kysely" => helpers::run_npx(&["kysely", "migrate:latest"], dir),
        "knex" => helpers::run_npx(&["knex", "migrate:latest"], dir),
        _ => helpers::run_npx(&["prisma", "migrate", "dev"], dir),
    }
}

fn orm_push(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "db", "push"], dir),
        "drizzle" => helpers::run_npx(&["drizzle-kit", "push"], dir),
        "typeorm" => helpers::run_npx(&["typeorm", "schema:sync"], dir),
        "mikroorm" => helpers::run_npx(&["mikro-orm", "schema:update", "--run"], dir),
        "sequelize" => helpers::run_npx(&["sequelize", "db:sync"], dir),
        _ => anyhow::bail!("Push not supported for {orm}"),
    }
}

fn orm_pull(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "db", "pull"], dir),
        "drizzle" => helpers::run_npx(&["drizzle-kit", "pull"], dir),
        "typeorm" => helpers::run_npx(&["typeorm", "model:sync"], dir),
        "mikroorm" => helpers::run_npx(&["mikro-orm", "schema:dump"], dir),
        _ => anyhow::bail!("Pull not supported for {orm}"),
    }
}

fn orm_seed(orm: &str, dir: &Path, _class: Option<&str>) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "db", "seed"], dir),
        "mikroorm" => helpers::run_npx(&["mikro-orm", "seeder:run"], dir),
        "sequelize" => helpers::run_npx(&["sequelize", "db:seed:all"], dir),
        "knex" => helpers::run_npx(&["knex", "seed:run"], dir),
        _ => anyhow::bail!("Seed not supported for {orm}"),
    }
}

fn orm_reset(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "migrate", "reset", "--force"], dir),
        "typeorm" => {
            helpers::run_npx(&["typeorm", "schema:drop"], dir)?;
            helpers::run_npx(&["typeorm", "migration:run"], dir)
        }
        "mikroorm" => {
            helpers::run_npx(&["mikro-orm", "schema:drop", "--run"], dir)?;
            helpers::run_npx(&["mikro-orm", "migration:up"], dir)
        }
        "sequelize" => helpers::run_npx(&["sequelize", "db:migrate:undo:all"], dir),
        "knex" => helpers::run_npx(&["knex", "migrate:rollback", "--all"], dir),
        _ => anyhow::bail!("Reset not supported for {orm}"),
    }
}

fn orm_studio(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "studio"], dir),
        "drizzle" => helpers::run_npx(&["drizzle-kit", "studio"], dir),
        _ => anyhow::bail!("Studio not supported for {orm}"),
    }
}

fn orm_rollback(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "migrate", "dev", "--create-only"], dir),
        "typeorm" => helpers::run_npx(&["typeorm", "migration:revert"], dir),
        "mikroorm" => helpers::run_npx(&["mikro-orm", "migration:down"], dir),
        "sequelize" => helpers::run_npx(&["sequelize", "db:migrate:undo"], dir),
        "knex" => helpers::run_npx(&["knex", "migrate:rollback"], dir),
        _ => helpers::run_npx(&["prisma", "migrate", "dev", "--create-only"], dir),
    }
}

fn orm_format(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "format"], dir),
        _ => anyhow::bail!("Format not supported for {orm}"),
    }
}

fn orm_validate(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "validate"], dir),
        _ => anyhow::bail!("Validate not supported for {orm}"),
    }
}

fn orm_create(orm: &str, dir: &Path, name: &str) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "migrate", "dev", "--name", name], dir),
        "drizzle" => helpers::run_npx(&["drizzle-kit", "generate", "--name", name], dir),
        "typeorm" => helpers::run_npx(&["typeorm", "migration:create", name], dir),
        "mikroorm" => helpers::run_npx(&["mikro-orm", "migration:create", name], dir),
        "sequelize" => helpers::run_npx(&["sequelize", "migration:generate", "--name", name], dir),
        "knex" => helpers::run_npx(&["knex", "migrate:make", name], dir),
        _ => anyhow::bail!("Create migration not supported for {orm}"),
    }
}


