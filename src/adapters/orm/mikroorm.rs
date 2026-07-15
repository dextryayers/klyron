use std::path::Path;
use crate::adapters::orm::{OrmTrait, OrmError};

pub struct MikroOrmAdapter;

impl OrmTrait for MikroOrmAdapter {
    fn name(&self) -> &'static str {
        "mikro-orm"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        let config_files = ["mikro-orm.config.ts", "mikro-orm.config.js"];
        if config_files.iter().any(|f| project_dir.join(f).exists()) {
            return true;
        }
        let pkg_path = project_dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(pkg_path) {
            if content.contains("\"mikro-orm\"") {
                return true;
            }
        }
        false
    }

    fn get_config_path(&self) -> &'static str {
        "mikro-orm.config.ts"
    }

    fn get_generate_command(&self) -> Vec<String> {
        vec!["npx".into(), "mikro-orm".into(), "schema:create".into()]
    }

    fn get_migrate_command(&self) -> Vec<String> {
        vec!["npx".into(), "mikro-orm".into(), "migration:up".into()]
    }

    fn get_seed_command(&self) -> Vec<String> {
        vec!["npx".into(), "mikro-orm".into(), "seeder:run".into()]
    }

    fn validate_schema(&self, project_dir: &Path) -> Result<Vec<String>, OrmError> {
        let config_found = ["mikro-orm.config.ts", "mikro-orm.config.js"]
            .iter().any(|f| project_dir.join(f).exists());
        if !config_found {
            return Err(OrmError::NotFound("mikro-orm.config.ts/js not found".into()));
        }
        let mut issues = Vec::new();
        let entities_dir = project_dir.join("src/entities");
        if !entities_dir.exists() {
            issues.push("Missing src/entities directory for entity files".into());
        }
        Ok(issues)
    }

    fn get_supported_databases(&self) -> Vec<&'static str> {
        vec!["postgresql", "mysql", "sqlite", "mongodb"]
    }
}
