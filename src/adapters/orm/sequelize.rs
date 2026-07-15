use std::path::Path;
use crate::adapters::orm::{OrmTrait, OrmError};

pub struct SequelizeAdapter;

impl OrmTrait for SequelizeAdapter {
    fn name(&self) -> &'static str {
        "sequelize"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        let config_files = ["config/config.json", ".sequelizerc"];
        if config_files.iter().any(|f| project_dir.join(f).exists()) {
            return true;
        }
        let pkg_path = project_dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(pkg_path) {
            if content.contains("\"sequelize\"") {
                return true;
            }
        }
        false
    }

    fn get_config_path(&self) -> &'static str {
        "config/config.json"
    }

    fn get_generate_command(&self) -> Vec<String> {
        vec!["npx".into(), "sequelize-cli".into(), "model:generate".into()]
    }

    fn get_migrate_command(&self) -> Vec<String> {
        vec!["npx".into(), "sequelize-cli".into(), "db:migrate".into()]
    }

    fn get_seed_command(&self) -> Vec<String> {
        vec!["npx".into(), "sequelize-cli".into(), "db:seed:all".into()]
    }

    fn validate_schema(&self, project_dir: &Path) -> Result<Vec<String>, OrmError> {
        let config_candidates = ["config/config.json", "config/config.js"];
        let config_found = config_candidates.iter().any(|f| project_dir.join(f).exists());
        if !config_found {
            return Err(OrmError::NotFound("No Sequelize config found (config/config.json or config/config.js)".into()));
        }
        let mut issues = Vec::new();
        let migrations_dir = project_dir.join("migrations");
        if !migrations_dir.exists() {
            issues.push("Missing migrations directory".into());
        }
        let models_dir = project_dir.join("models");
        if !models_dir.exists() {
            issues.push("Missing models directory".into());
        }
        Ok(issues)
    }

    fn get_supported_databases(&self) -> Vec<&'static str> {
        vec!["postgresql", "mysql", "sqlite", "mariadb", "mssql"]
    }
}
