use crate::commands::helpers;
use clap::Subcommand;
use std::path::Path;

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
            let detected = helpers::detect_orm(&dir);
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

fn orm_init_direct(orm: &str, dir: &Path) -> anyhow::Result<()> {
    match orm {
        "prisma" => helpers::run_npx(&["prisma", "init"], dir),
        "drizzle" => helpers::run_npx(&["drizzle-kit", "init"], dir),
        "typeorm" => helpers::run_npx(&["typeorm", "init"], dir),
        "mikroorm" => helpers::run_npx(&["mikro-orm", "init"], dir),
        "sequelize" => helpers::run_npx(&["sequelize", "cli", "init"], dir),
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
        "knex" => helpers::run_npx(&["knex", "init"], dir),
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
        PrismaAction::Init => helpers::run_npx(&["prisma", "init"], &dir),
        PrismaAction::Generate => helpers::run_npx(&["prisma", "generate"], &dir),
        PrismaAction::Migrate { name } => {
            if let Some(n) = name { helpers::run_npx(&["prisma", "migrate", "dev", "--name", &n], &dir) }
            else { helpers::run_npx(&["prisma", "migrate", "dev"], &dir) }
        }
        PrismaAction::Studio => helpers::run_npx(&["prisma", "studio"], &dir),
        PrismaAction::DbPush => helpers::run_npx(&["prisma", "db", "push"], &dir),
        PrismaAction::DbPull => helpers::run_npx(&["prisma", "db", "pull"], &dir),
        PrismaAction::Seed { .. } => {
            helpers::run_npx(&["prisma", "db", "seed"], &dir)
        }
        PrismaAction::Reset => helpers::run_npx(&["prisma", "migrate", "reset", "--force"], &dir),
        PrismaAction::Rollback { step: _ } => {
            helpers::run_npx(&["prisma", "migrate", "dev"], &dir)
        }
        PrismaAction::Format => helpers::run_npx(&["prisma", "format"], &dir),
        PrismaAction::Validate => helpers::run_npx(&["prisma", "validate"], &dir),
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
        DrizzleAction::Init => helpers::run_npx(&["drizzle-kit", "init"], &dir),
        DrizzleAction::Generate => helpers::run_npx(&["drizzle-kit", "generate"], &dir),
        DrizzleAction::Migrate => helpers::run_npx(&["drizzle-kit", "migrate"], &dir),
        DrizzleAction::Studio => helpers::run_npx(&["drizzle-kit", "studio"], &dir),
        DrizzleAction::Push => helpers::run_npx(&["drizzle-kit", "push"], &dir),
        DrizzleAction::Pull => helpers::run_npx(&["drizzle-kit", "pull"], &dir),
        DrizzleAction::Check => helpers::run_npx(&["drizzle-kit", "check"], &dir),
        DrizzleAction::Up => helpers::run_npx(&["drizzle-kit", "up"], &dir),
        DrizzleAction::Down => helpers::run_npx(&["drizzle-kit", "down"], &dir),
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
    Seed,
    Studio,
    Pull,
}

pub fn run_typeorm(action: TypeOrmAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        TypeOrmAction::Init => helpers::run_npx(&["typeorm", "init"], &dir),
        TypeOrmAction::MigrateGenerate { name } => helpers::run_npx(&["typeorm", "migration:generate", "--name", &name], &dir),
        TypeOrmAction::MigrateRun => helpers::run_npx(&["typeorm", "migration:run"], &dir),
        TypeOrmAction::MigrateRevert => helpers::run_npx(&["typeorm", "migration:revert"], &dir),
        TypeOrmAction::SchemaSync => helpers::run_npx(&["typeorm", "schema:sync"], &dir),
        TypeOrmAction::SchemaDrop => helpers::run_npx(&["typeorm", "schema:drop"], &dir),
        TypeOrmAction::MigrationCreate { name } => helpers::run_npx(&["typeorm", "migration:create", &name], &dir),
        TypeOrmAction::Seed => {
            println!("TypeORM does not have a built-in seed command.");
            println!("Use `typeorm-seeding` package or write a custom seed script.");
            Ok(())
        }
        TypeOrmAction::Studio => {
            println!("TypeORM does not have a built-in studio command.");
            println!("Use a GUI tool like DBeaver, TablePlus, or pgAdmin.");
            Ok(())
        }
        TypeOrmAction::Pull => {
            helpers::run_npx(&["typeorm", "model:sync"], &dir)
        }
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
        MikroOrmAction::Init => helpers::run_npx(&["mikro-orm", "init"], &dir),
        MikroOrmAction::MigrationCreate { name } => helpers::run_npx(&["mikro-orm", "migration:create", &name], &dir),
        MikroOrmAction::MigrationUp => helpers::run_npx(&["mikro-orm", "migration:up"], &dir),
        MikroOrmAction::MigrationDown => helpers::run_npx(&["mikro-orm", "migration:down"], &dir),
        MikroOrmAction::MigrationList => helpers::run_npx(&["mikro-orm", "migration:list"], &dir),
        MikroOrmAction::SchemaUpdate => helpers::run_npx(&["mikro-orm", "schema:update", "--run"], &dir),
        MikroOrmAction::SchemaDrop => helpers::run_npx(&["mikro-orm", "schema:drop", "--run"], &dir),
        MikroOrmAction::SeederRun => helpers::run_npx(&["mikro-orm", "seeder:run"], &dir),
        MikroOrmAction::SeederCreate { name } => helpers::run_npx(&["mikro-orm", "seeder:create", &name], &dir),
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
    Push,
}

pub fn run_sequelize(action: SequelizeAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        SequelizeAction::Init => helpers::run_npx(&["sequelize", "init"], &dir),
        SequelizeAction::MigrateGenerate { name } => helpers::run_npx(&["sequelize", "migration:generate", "--name", &name], &dir),
        SequelizeAction::MigrateRun => helpers::run_npx(&["sequelize", "db:migrate"], &dir),
        SequelizeAction::MigrateUndo => helpers::run_npx(&["sequelize", "db:migrate:undo"], &dir),
        SequelizeAction::MigrateUndoAll => helpers::run_npx(&["sequelize", "db:migrate:undo:all"], &dir),
        SequelizeAction::SeedGenerate { name } => helpers::run_npx(&["sequelize", "seed:generate", "--name", &name], &dir),
        SequelizeAction::SeedRun => helpers::run_npx(&["sequelize", "db:seed:all"], &dir),
        SequelizeAction::SeedUndo => helpers::run_npx(&["sequelize", "db:seed:undo"], &dir),
        SequelizeAction::DbCreate => helpers::run_npx(&["sequelize", "db:create"], &dir),
        SequelizeAction::DbDrop => helpers::run_npx(&["sequelize", "db:drop"], &dir),
        SequelizeAction::Push => helpers::run_npx(&["sequelize", "db:migrate"], &dir),
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
            let use_ts = dir.join("tsconfig.json").exists();
            let ext = if use_ts { "ts" } else { "js" };
            let filename = format!("models/{}.{}", name.to_lowercase(), ext);
            let model_path = dir.join(&filename);
            std::fs::create_dir_all(model_path.parent().unwrap())?;
            let content = if use_ts {
                format!("import mongoose, {{ Schema, Document, Model }} from 'mongoose';\n\ninterface I{name} extends Document {{\n  name: string;\n  createdAt: Date;\n}}\n\nconst {name}Schema = new Schema<I{name}>({{\n  name: {{ type: String, required: true }},\n  createdAt: {{ type: Date, default: Date.now }},\n}});\n\nconst {name}: Model<I{name}> = mongoose.model<I{name}>('{name}', {name}Schema);\nexport default {name};\n")
            } else {
                format!("const mongoose = require('mongoose');\nconst {{ Schema }} = mongoose;\n\nconst {name}Schema = new Schema({{\n  name: {{ type: String, required: true }},\n  createdAt: {{ type: Date, default: Date.now }},\n}});\n\nmodule.exports = mongoose.model('{name}', {name}Schema);\n")
            };
            std::fs::write(&model_path, content)?;
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
        KyselyAction::Generate => helpers::run_npx(&["kysely", "generate"], &dir),
        KyselyAction::MigrateLatest => helpers::run_npx(&["kysely", "migrate:latest"], &dir),
        KyselyAction::MigrateUp => helpers::run_npx(&["kysely", "migrate:up"], &dir),
        KyselyAction::MigrateDown => helpers::run_npx(&["kysely", "migrate:down"], &dir),
        KyselyAction::MigrateList => helpers::run_npx(&["kysely", "migrate:list"], &dir),
        KyselyAction::SeedRun => helpers::run_npx(&["kysely", "seed:run"], &dir),
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
    Studio,
    Pull,
}

pub fn run_knex(action: KnexAction) -> anyhow::Result<()> {
    let dir = std::env::current_dir()?;
    match action {
        KnexAction::Init => helpers::run_npx(&["knex", "init"], &dir),
        KnexAction::MigrateMake { name } => helpers::run_npx(&["knex", "migrate:make", &name], &dir),
        KnexAction::MigrateLatest => helpers::run_npx(&["knex", "migrate:latest"], &dir),
        KnexAction::MigrateUp => helpers::run_npx(&["knex", "migrate:up"], &dir),
        KnexAction::MigrateDown => helpers::run_npx(&["knex", "migrate:down"], &dir),
        KnexAction::MigrateRollback => helpers::run_npx(&["knex", "migrate:rollback"], &dir),
        KnexAction::MigrateList => helpers::run_npx(&["knex", "migrate:list"], &dir),
        KnexAction::SeedMake { name } => helpers::run_npx(&["knex", "seed:make", &name], &dir),
        KnexAction::SeedRun => helpers::run_npx(&["knex", "seed:run"], &dir),
        KnexAction::Studio => {
            println!("knex-admin provides a GUI for Knex. Install it with:");
            println!("  npm install knex-admin");
            println!("Then run: npx knex-admin");
            Ok(())
        }
        KnexAction::Pull => {
            println!("Knex pull is available via plugins like knex-pull or knex-db-pull.");
            println!("Install with: npm install knex-pull");
            println!("Then run: npx knex-pull");
            Ok(())
        }
    }
}


