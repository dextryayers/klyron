use std::collections::HashMap;
use std::path::Path;

pub mod prisma;
pub mod drizzle;
pub mod typeorm;
pub mod sequelize;
pub mod mikroorm;
pub mod mongoose;
pub mod knex;
pub mod kysely;

pub trait OrmTrait: Send + Sync {
    fn name(&self) -> &'static str;
    fn detect(&self, project_dir: &Path) -> bool;
    fn get_config_path(&self) -> &'static str;
    fn get_generate_command(&self) -> Vec<String>;
    fn get_migrate_command(&self) -> Vec<String>;
    fn get_seed_command(&self) -> Vec<String>;
    fn validate_schema(&self, project_dir: &Path) -> Result<Vec<String>, OrmError>;
    fn get_supported_databases(&self) -> Vec<&'static str>;
}

#[derive(Debug)]
pub enum OrmError {
    NotFound(String),
    SchemaError(String),
    MigrationError(String),
}

impl std::fmt::Display for OrmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrmError::NotFound(s) => write!(f, "ORM not found: {s}"),
            OrmError::SchemaError(s) => write!(f, "Schema error: {s}"),
            OrmError::MigrationError(s) => write!(f, "Migration error: {s}"),
        }
    }
}

impl std::error::Error for OrmError {}

pub struct OrmRegistry {
    orms: HashMap<String, Box<dyn OrmTrait>>,
}

impl OrmRegistry {
    pub fn new() -> Self {
        Self {
            orms: HashMap::new(),
        }
    }

    pub fn register(&mut self, orm: Box<dyn OrmTrait>) {
        self.orms.insert(orm.name().to_string(), orm);
    }

    pub fn get(&self, name: &str) -> Option<&Box<dyn OrmTrait>> {
        self.orms.get(name)
    }

    pub fn list(&self) -> Vec<&str> {
        self.orms.keys().map(|s| s.as_str()).collect()
    }

    pub fn detect(&self, project_dir: &Path) -> Option<&Box<dyn OrmTrait>> {
        for orm in self.orms.values() {
            if orm.detect(project_dir) {
                return Some(orm);
            }
        }
        None
    }

    pub fn register_all(&mut self) {
        self.register(Box::new(prisma::PrismaAdapter));
        self.register(Box::new(drizzle::DrizzleAdapter));
        self.register(Box::new(typeorm::TypeOrmAdapter));
        self.register(Box::new(sequelize::SequelizeAdapter));
        self.register(Box::new(mikroorm::MikroOrmAdapter));
        self.register(Box::new(mongoose::MongooseAdapter));
        self.register(Box::new(knex::KnexAdapter));
        self.register(Box::new(kysely::KyselyAdapter));
    }
}

impl Default for OrmRegistry {
    fn default() -> Self {
        Self::new()
    }
}
