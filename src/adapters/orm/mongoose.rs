use std::path::Path;
use crate::adapters::orm::{OrmTrait, OrmError};

pub struct MongooseAdapter;

impl OrmTrait for MongooseAdapter {
    fn name(&self) -> &'static str {
        "mongoose"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        let model_dirs = ["src/models", "models"];
        for dir in &model_dirs {
            let models_path = project_dir.join(dir);
            if models_path.exists() && models_path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&models_path) {
                    if entries.filter_map(|e| e.ok()).any(|e| e.path().is_file()) {
                        return true;
                    }
                }
            }
        }
        let pkg_path = project_dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(pkg_path) {
            if content.contains("\"mongoose\"") {
                return true;
            }
        }
        false
    }

    fn get_config_path(&self) -> &'static str {
        "src/config/database.ts"
    }

    fn get_generate_command(&self) -> Vec<String> {
        vec![]
    }

    fn get_migrate_command(&self) -> Vec<String> {
        vec![]
    }

    fn get_seed_command(&self) -> Vec<String> {
        vec!["npx".into(), "tsx".into(), "src/seed.ts".into()]
    }

    fn validate_schema(&self, project_dir: &Path) -> Result<Vec<String>, OrmError> {
        let mut issues = Vec::new();
        let model_dirs = ["src/models", "models"];
        let models_found = model_dirs.iter().any(|d| {
            let p = project_dir.join(d);
            p.exists() && p.is_dir()
        });
        if !models_found {
            issues.push("Missing models directory (src/models or models)".into());
        }
        let config_files = ["src/config/database.ts", "src/db/connect.ts", "src/db.ts"];
        let config_found = config_files.iter().any(|f| project_dir.join(f).exists());
        if !config_found {
            issues.push("Missing database config file (src/config/database.ts or src/db/connect.ts)".into());
        }
        Ok(issues)
    }

    fn get_supported_databases(&self) -> Vec<&'static str> {
        vec!["mongodb"]
    }
}
