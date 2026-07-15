use std::path::Path;
use crate::adapters::orm::{OrmTrait, OrmError};

pub struct KnexAdapter;

impl OrmTrait for KnexAdapter {
    fn name(&self) -> &'static str {
        "knex"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        let config_files = ["knexfile.ts", "knexfile.js", "src/knexfile.ts", "src/knexfile.js"];
        if config_files.iter().any(|f| project_dir.join(f).exists()) {
            return true;
        }
        let pkg_path = project_dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(pkg_path) {
            if content.contains("\"knex\"") {
                return true;
            }
        }
        false
    }

    fn get_config_path(&self) -> &'static str {
        "knexfile.ts"
    }

    fn get_generate_command(&self) -> Vec<String> {
        vec!["npx".into(), "knex".into(), "migrate:make".into()]
    }

    fn get_migrate_command(&self) -> Vec<String> {
        vec!["npx".into(), "knex".into(), "migrate:latest".into()]
    }

    fn get_seed_command(&self) -> Vec<String> {
        vec!["npx".into(), "knex".into(), "seed:run".into()]
    }

    fn validate_schema(&self, project_dir: &Path) -> Result<Vec<String>, OrmError> {
        let config_found = ["knexfile.ts", "knexfile.js", "src/knexfile.ts", "src/knexfile.js"]
            .iter().any(|f| project_dir.join(f).exists());
        if !config_found {
            return Err(OrmError::NotFound("No knexfile found (knexfile.ts/js or src/knexfile.ts/js)".into()));
        }
        let mut issues = Vec::new();
        let migrations_dir = project_dir.join("migrations");
        if !migrations_dir.exists() {
            issues.push("Missing migrations directory".into());
        }
        Ok(issues)
    }

    fn get_supported_databases(&self) -> Vec<&'static str> {
        vec!["postgresql", "mysql", "sqlite3", "oracledb", "mssql", "better-sqlite3"]
    }
}
