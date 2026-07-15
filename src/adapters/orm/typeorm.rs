use std::path::Path;
use crate::adapters::orm::{OrmTrait, OrmError};

pub struct TypeOrmAdapter;

impl OrmTrait for TypeOrmAdapter {
    fn name(&self) -> &'static str {
        "typeorm"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        let config_files = [
            "ormconfig.json", "ormconfig.ts", "ormconfig.js",
            "src/data-source.ts", "src/data-source.js",
            "app-data-source.ts", "app-data-source.js",
        ];
        if config_files.iter().any(|f| project_dir.join(f).exists()) {
            return true;
        }
        let pkg_path = project_dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(pkg_path) {
            if content.contains("\"typeorm\"") {
                return true;
            }
        }
        false
    }

    fn get_config_path(&self) -> &'static str {
        "src/data-source.ts"
    }

    fn get_generate_command(&self) -> Vec<String> {
        vec!["npx".into(), "typeorm".into(), "migration:generate".into()]
    }

    fn get_migrate_command(&self) -> Vec<String> {
        vec!["npx".into(), "typeorm".into(), "migration:run".into()]
    }

    fn get_seed_command(&self) -> Vec<String> {
        vec!["npx".into(), "ts-node".into(), "src/seed.ts".into()]
    }

    fn validate_schema(&self, project_dir: &Path) -> Result<Vec<String>, OrmError> {
        let candidates = [
            "ormconfig.json", "ormconfig.ts", "ormconfig.js",
            "src/data-source.ts", "src/data-source.js",
        ];
        let config_found = candidates.iter().any(|f| project_dir.join(f).exists());
        if !config_found {
            return Err(OrmError::NotFound("No TypeORM configuration found (ormconfig.* or data-source.*)".into()));
        }
        let mut issues = Vec::new();
        let entities_dir = project_dir.join("src/entity");
        if !entities_dir.exists() {
            issues.push("Missing src/entity directory for entity files".into());
        }
        Ok(issues)
    }

    fn get_supported_databases(&self) -> Vec<&'static str> {
        vec!["postgresql", "mysql", "sqlite", "mariadb", "cockroachdb", "better-sqlite3"]
    }
}
