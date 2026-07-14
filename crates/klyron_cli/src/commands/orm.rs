use clap::Subcommand;
use std::path::Path;
use std::process::Command;

/// Unified ORM command namespace
#[derive(Subcommand)]
pub enum OrmCommand {
    /// List all supported ORMs and detect current project
    List,
    /// Initialize ORM config (alias for `db init --orm <orm>`)
    Init {
        /// ORM to initialize (prisma, drizzle, typeorm, mikroorm, sequelize, mongoose, kysely, knex)
        orm: Option<String>,
    },
}

pub fn run_orm(cmd: OrmCommand) -> anyhow::Result<()> {
    match cmd {
        OrmCommand::List => {
            println!("Supported ORMs:");
            println!("  prisma     - Prisma (schema.prisma)");
            println!("  drizzle    - Drizzle ORM (drizzle.config.ts)");
            println!("  typeorm    - TypeORM (ormconfig.json)");
            println!("  mikroorm   - MikroORM (mikro-orm.config.ts)");
            println!("  sequelize  - Sequelize (config/config.json)");
            println!("  mongoose   - Mongoose (models/)");
            println!("  kysely     - Kysely (kysely.ts)");
            println!("  knex       - Knex (knexfile.ts)");
            let dir = std::env::current_dir().unwrap_or_default();
            let detected = detect_orm_in_dir(&dir);
            if detected != "unknown" {
                println!("\nDetected ORM in current directory: {detected}");
            } else {
                println!("\nNo ORM detected in current directory.");
            }
            Ok(())
        }
        OrmCommand::Init { orm } => {
            let dir = std::env::current_dir()?;
            let orm_name = orm.as_deref().unwrap_or("prisma");
            orm_init_direct(orm_name, &dir)
        }
    }
}

fn detect_orm_in_dir(dir: &Path) -> String {
    if dir.join("schema.prisma").exists() || dir.join("prisma/schema.prisma").exists() { return "prisma".into(); }
    if dir.join("drizzle.config.ts").exists() || dir.join("drizzle.config.js").exists() { return "drizzle".into(); }
    if dir.join("ormconfig.json").exists() || dir.join("ormconfig.ts").exists() { return "typeorm".into(); }
    if dir.join("mikro-orm.config.ts").exists() || dir.join("mikro-orm.config.js").exists() { return "mikroorm".into(); }
    if dir.join("knexfile.ts").exists() || dir.join("knexfile.js").exists() { return "knex".into(); }
    if dir.join("kysely.ts").exists() { return "kysely".into(); }
    "unknown".to_string()
}

fn orm_init_direct(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => run_npx(&["prisma", "init"], dir),
        "drizzle" => run_npx(&["drizzle-kit", "init"], dir),
        "typeorm" => run_npx(&["typeorm", "init"], dir),
        "mikroorm" => run_npx(&["mikro-orm", "init"], dir),
        "sequelize" => run_npx(&["sequelize", "cli", "init"], dir),
        "mongoose" => {
            std::fs::create_dir_all(dir.join("models"))?;
            std::fs::write(dir.join("models/index.js"),
                r#"const mongoose = require('mongoose');
mongoose.connect(process.env.MONGO_URI || 'mongodb://localhost:27017/myapp');
module.exports = mongoose;
"#)?;
            println!("Mongoose setup complete. Add schemas to models/");
            Ok(())
        }
        "kysely" => {
            std::fs::write(dir.join("kysely.ts"),
                r#"import { Kysely, PostgresDialect, Generated } from 'kysely';
import { Pool } from 'pg';
interface Database { person: PersonTable; }
interface PersonTable { id: Generated<number>; first_name: string; last_name: string; }
const db = new Kysely<Database>({
  dialect: new PostgresDialect({ pool: new Pool({ connectionString: process.env.DATABASE_URL }) }),
});
export { db };"#)?;
            Ok(())
        }
        "knex" => run_npx(&["knex", "init"], dir),
        _ => anyhow::bail!("Unknown ORM: {orm}. Supported: prisma, drizzle, typeorm, mikroorm, sequelize, mongoose, kysely, knex"),
    }
}

// ── Prisma ─────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum PrismaAction {
    Init,
    Generate,
    Migrate { #[arg(short, long)] name: Option<String> },
    Studio,
    DbPush,
    DbPull,
    Seed { #[arg(short, long)] class: Option<String> },
    Reset,
    Rollback { #[arg(short, long)] step: Option<usize> },
    Format,
    Validate,
}

pub fn run_prisma(action: PrismaAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        PrismaAction::Init => run_npx(&["prisma", "init"], &dir),
        PrismaAction::Generate => run_npx(&["prisma", "generate"], &dir),
        PrismaAction::Migrate { name } => {
            if let Some(n) = name { run_npx(&["prisma", "migrate", "dev", "--name", &n], &dir) }
            else { run_npx(&["prisma", "migrate", "dev"], &dir) }
        }
        PrismaAction::Studio => run_npx(&["prisma", "studio"], &dir),
        PrismaAction::DbPush => run_npx(&["prisma", "db", "push"], &dir),
        PrismaAction::DbPull => run_npx(&["prisma", "db", "pull"], &dir),
        PrismaAction::Seed { class } => {
            if let Some(c) = class { run_npx(&["prisma", "db", "seed", "--", "--class", &c], &dir) }
            else { run_npx(&["prisma", "db", "seed"], &dir) }
        }
        PrismaAction::Reset => run_npx(&["prisma", "migrate", "reset", "--force"], &dir),
        PrismaAction::Rollback { step: _ } => run_npx(&["prisma", "migrate", "dev", "--create-only"], &dir),
        PrismaAction::Format => run_npx(&["prisma", "format"], &dir),
        PrismaAction::Validate => run_npx(&["prisma", "validate"], &dir),
    }
}

// ── Drizzle ────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum DrizzleAction {
    Init,
    Generate,
    Migrate,
    Studio,
    Push,
    Pull,
    Check,
    Up,
    Down,
}

pub fn run_drizzle(action: DrizzleAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        DrizzleAction::Init => run_npx(&["drizzle-kit", "init"], &dir),
        DrizzleAction::Generate => run_npx(&["drizzle-kit", "generate"], &dir),
        DrizzleAction::Migrate => run_npx(&["drizzle-kit", "migrate"], &dir),
        DrizzleAction::Studio => run_npx(&["drizzle-kit", "studio"], &dir),
        DrizzleAction::Push => run_npx(&["drizzle-kit", "push"], &dir),
        DrizzleAction::Pull => run_npx(&["drizzle-kit", "pull"], &dir),
        DrizzleAction::Check => run_npx(&["drizzle-kit", "check"], &dir),
        DrizzleAction::Up => run_npx(&["drizzle-kit", "up"], &dir),
        DrizzleAction::Down => run_npx(&["drizzle-kit", "down"], &dir),
    }
}

// ── TypeORM ────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum TypeOrmAction {
    Init,
    MigrateGenerate { name: String },
    MigrateRun,
    MigrateRevert,
    SchemaSync,
    SchemaDrop,
    MigrationCreate { name: String },
}

pub fn run_typeorm(action: TypeOrmAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        TypeOrmAction::Init => run_npx(&["typeorm", "init"], &dir),
        TypeOrmAction::MigrateGenerate { name } => run_npx(&["typeorm", "migration:generate", "-d", &name], &dir),
        TypeOrmAction::MigrateRun => run_npx(&["typeorm", "migration:run"], &dir),
        TypeOrmAction::MigrateRevert => run_npx(&["typeorm", "migration:revert"], &dir),
        TypeOrmAction::SchemaSync => run_npx(&["typeorm", "schema:sync"], &dir),
        TypeOrmAction::SchemaDrop => run_npx(&["typeorm", "schema:drop"], &dir),
        TypeOrmAction::MigrationCreate { name } => run_npx(&["typeorm", "migration:create", &name], &dir),
    }
}

// ── MikroORM ───────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum MikroOrmAction {
    Init,
    MigrationCreate { name: String },
    MigrationUp,
    MigrationDown,
    MigrationList,
    SchemaUpdate,
    SchemaDrop,
    SeederRun,
    SeederCreate { name: String },
}

pub fn run_mikroorm(action: MikroOrmAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        MikroOrmAction::Init => run_npx(&["mikro-orm", "init"], &dir),
        MikroOrmAction::MigrationCreate { name } => run_npx(&["mikro-orm", "migration:create", &name], &dir),
        MikroOrmAction::MigrationUp => run_npx(&["mikro-orm", "migration:up"], &dir),
        MikroOrmAction::MigrationDown => run_npx(&["mikro-orm", "migration:down"], &dir),
        MikroOrmAction::MigrationList => run_npx(&["mikro-orm", "migration:list"], &dir),
        MikroOrmAction::SchemaUpdate => run_npx(&["mikro-orm", "schema:update", "--run"], &dir),
        MikroOrmAction::SchemaDrop => run_npx(&["mikro-orm", "schema:drop", "--run"], &dir),
        MikroOrmAction::SeederRun => run_npx(&["mikro-orm", "seeder:run"], &dir),
        MikroOrmAction::SeederCreate { name } => run_npx(&["mikro-orm", "seeder:create", &name], &dir),
    }
}

// ── Sequelize ──────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum SequelizeAction {
    Init,
    MigrateGenerate { name: String },
    MigrateRun,
    MigrateUndo,
    MigrateUndoAll,
    SeedGenerate { name: String },
    SeedRun,
    SeedUndo,
    DbCreate,
    DbDrop,
}

pub fn run_sequelize(action: SequelizeAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        SequelizeAction::Init => run_npx(&["sequelize", "init"], &dir),
        SequelizeAction::MigrateGenerate { name } => run_npx(&["sequelize", "migration:generate", "--name", &name], &dir),
        SequelizeAction::MigrateRun => run_npx(&["sequelize", "db:migrate"], &dir),
        SequelizeAction::MigrateUndo => run_npx(&["sequelize", "db:migrate:undo"], &dir),
        SequelizeAction::MigrateUndoAll => run_npx(&["sequelize", "db:migrate:undo:all"], &dir),
        SequelizeAction::SeedGenerate { name } => run_npx(&["sequelize", "seed:generate", "--name", &name], &dir),
        SequelizeAction::SeedRun => run_npx(&["sequelize", "db:seed:all"], &dir),
        SequelizeAction::SeedUndo => run_npx(&["sequelize", "db:seed:undo"], &dir),
        SequelizeAction::DbCreate => run_npx(&["sequelize", "db:create"], &dir),
        SequelizeAction::DbDrop => run_npx(&["sequelize", "db:drop"], &dir),
    }
}

// ── Mongoose ───────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum MongooseAction {
    Init,
    Generate { name: String },
}

pub fn run_mongoose(action: MongooseAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        MongooseAction::Init => {
            println!("Mongoose does not have an official CLI scaffold.");
            println!("Creating mongoose schema directory...");
            std::fs::create_dir_all(dir.join("models"))?;
            std::fs::write(dir.join("models/index.js"),
                r#"const mongoose = require('mongoose');
mongoose.connect(process.env.MONGO_URI || 'mongodb://localhost:27017/myapp');
module.exports = mongoose;
"#)?;
            println!("Mongoose setup complete. Add your schemas to models/");
            Ok(())
        }
        MongooseAction::Generate { name } => {
            let filename = format!("models/{}.js", name.to_lowercase());
            let model_path = dir.join(&filename);
            std::fs::create_dir_all(model_path.parent().unwrap())?;
            std::fs::write(&model_path,
                format!("const mongoose = require('mongoose');\nconst {{ Schema }} = mongoose;\n\nconst {name}Schema = new Schema({{\n  name: {{ type: String, required: true }},\n  createdAt: {{ type: Date, default: Date.now }},\n}});\n\nmodule.exports = mongoose.model('{name}', {name}Schema);\n"))?;
            println!("Mongoose model created: {}", model_path.display());
            Ok(())
        }
    }
}

// ── Kysely ─────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum KyselyAction {
    Init,
    Generate,
    MigrateLatest,
    MigrateUp,
    MigrateDown,
    MigrateList,
    SeedRun,
}

pub fn run_kysely(action: KyselyAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        KyselyAction::Init => {
            println!("Kysely setup: create a `kysely.ts` config file with DB connection.");
            std::fs::write(dir.join("kysely.ts"),
                r#"import { Kysely, PostgresDialect, Generated, ColumnType } from 'kysely';
import { Pool } from 'pg';

interface Database {
  person: PersonTable;
}

interface PersonTable {
  id: Generated<number>;
  first_name: string;
  last_name: string;
  created_at: ColumnType<Date, string | undefined, never>;
}

const db = new Kysely<Database>({
  dialect: new PostgresDialect({
    pool: new Pool({
      connectionString: process.env.DATABASE_URL || 'postgres://localhost/myapp',
    }),
  }),
});

export { db, Database };
export type { PersonTable };
"#)?;
            println!("kysely.ts created. Add your schema types.");
            Ok(())
        }
        KyselyAction::Generate => run_npx(&["kysely", "generate"], &dir),
        KyselyAction::MigrateLatest => run_npx(&["kysely", "migrate:latest"], &dir),
        KyselyAction::MigrateUp => run_npx(&["kysely", "migrate:up"], &dir),
        KyselyAction::MigrateDown => run_npx(&["kysely", "migrate:down"], &dir),
        KyselyAction::MigrateList => run_npx(&["kysely", "migrate:list"], &dir),
        KyselyAction::SeedRun => run_npx(&["kysely", "seed:run"], &dir),
    }
}

// ── Knex ───────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum KnexAction {
    Init,
    MigrateMake { name: String },
    MigrateLatest,
    MigrateUp,
    MigrateDown,
    MigrateRollback,
    MigrateList,
    SeedMake { name: String },
    SeedRun,
}

pub fn run_knex(action: KnexAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        KnexAction::Init => run_npx(&["knex", "init"], &dir),
        KnexAction::MigrateMake { name } => run_npx(&["knex", "migrate:make", &name], &dir),
        KnexAction::MigrateLatest => run_npx(&["knex", "migrate:latest"], &dir),
        KnexAction::MigrateUp => run_npx(&["knex", "migrate:up"], &dir),
        KnexAction::MigrateDown => run_npx(&["knex", "migrate:down"], &dir),
        KnexAction::MigrateRollback => run_npx(&["knex", "migrate:rollback"], &dir),
        KnexAction::MigrateList => run_npx(&["knex", "migrate:list"], &dir),
        KnexAction::SeedMake { name } => run_npx(&["knex", "seed:make", &name], &dir),
        KnexAction::SeedRun => run_npx(&["knex", "seed:run"], &dir),
    }
}

// ── Shared helpers ─────────────────────────────────────────────────────────

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
