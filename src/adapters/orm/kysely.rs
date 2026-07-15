use std::path::Path;
use crate::adapters::orm::{OrmTrait, OrmError};

pub struct KyselyAdapter;

impl OrmTrait for KyselyAdapter {
    fn name(&self) -> &'static str {
        "kysely"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        let config_files = ["kysely.config.ts", "kysely.config.js"];
        if config_files.iter().any(|f| project_dir.join(f).exists()) {
            return true;
        }
        let type_files = ["src/db/types.ts", "src/db/types.d.ts"];
        if type_files.iter().any(|f| project_dir.join(f).exists()) {
            return true;
        }
        let pkg_path = project_dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(pkg_path) {
            if content.contains("\"kysely\"") {
                return true;
            }
        }
        false
    }

    fn get_config_path(&self) -> &'static str {
        "kysely.config.ts"
    }

    fn get_generate_command(&self) -> Vec<String> {
        vec!["npx".into(), "kysely".into(), "codegen".into()]
    }

    fn get_migrate_command(&self) -> Vec<String> {
        vec!["npx".into(), "kysely".into(), "migrate:latest".into()]
    }

    fn get_seed_command(&self) -> Vec<String> {
        vec!["npx".into(), "kysely".into(), "seed:run".into()]
    }

    fn validate_schema(&self, project_dir: &Path) -> Result<Vec<String>, OrmError> {
        let config_found = ["kysely.config.ts", "kysely.config.js"]
            .iter().any(|f| project_dir.join(f).exists());
        if !config_found {
            return Err(OrmError::NotFound("kysely.config.ts/js not found".into()));
        }
        let mut issues = Vec::new();
        let migrations_dir = project_dir.join("src/migrations");
        if !migrations_dir.exists() {
            issues.push("Missing src/migrations directory".into());
        }
        Ok(issues)
    }

    fn get_supported_databases(&self) -> Vec<&'static str> {
        vec!["postgresql", "mysql", "sqlite"]
    }
}
