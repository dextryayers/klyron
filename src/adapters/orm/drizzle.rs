use std::path::Path;
use crate::adapters::orm::{OrmTrait, OrmError};

pub struct DrizzleAdapter;

impl OrmTrait for DrizzleAdapter {
    fn name(&self) -> &'static str {
        "drizzle"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        if project_dir.join("drizzle.config.ts").exists() || project_dir.join("drizzle.config.js").exists() {
            return true;
        }
        let pkg_path = project_dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(pkg_path) {
            if content.contains("\"drizzle-orm\"") {
                return true;
            }
        }
        false
    }

    fn get_config_path(&self) -> &'static str {
        "drizzle.config.ts"
    }

    fn get_generate_command(&self) -> Vec<String> {
        vec!["npx".into(), "drizzle-kit".into(), "generate".into()]
    }

    fn get_migrate_command(&self) -> Vec<String> {
        vec!["npx".into(), "drizzle-kit".into(), "migrate".into()]
    }

    fn get_seed_command(&self) -> Vec<String> {
        vec!["npx".into(), "tsx".into(), "src/db/seed.ts".into()]
    }

    fn validate_schema(&self, project_dir: &Path) -> Result<Vec<String>, OrmError> {
        let candidates = ["drizzle.config.ts", "drizzle.config.js"];
        let config_found = candidates.iter().any(|f| project_dir.join(f).exists());
        if !config_found {
            return Err(OrmError::NotFound("drizzle.config.ts/js not found".into()));
        }
        let mut issues = Vec::new();
        let schema_dir = project_dir.join("src/db");
        if !schema_dir.exists() {
            issues.push("Missing src/db directory for schema files".into());
        }
        Ok(issues)
    }

    fn get_supported_databases(&self) -> Vec<&'static str> {
        vec!["postgresql", "mysql", "sqlite", "turso"]
    }
}
