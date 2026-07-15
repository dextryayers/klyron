use std::path::Path;
use crate::adapters::orm::{OrmTrait, OrmError};

pub struct PrismaAdapter;

impl OrmTrait for PrismaAdapter {
    fn name(&self) -> &'static str {
        "prisma"
    }

    fn detect(&self, project_dir: &Path) -> bool {
        if project_dir.join("prisma/schema.prisma").exists() {
            return true;
        }
        let pkg_path = project_dir.join("package.json");
        if let Ok(content) = std::fs::read_to_string(pkg_path) {
            if content.contains("\"prisma\"") {
                return true;
            }
        }
        false
    }

    fn get_config_path(&self) -> &'static str {
        "prisma/schema.prisma"
    }

    fn get_generate_command(&self) -> Vec<String> {
        vec!["npx".into(), "prisma".into(), "generate".into()]
    }

    fn get_migrate_command(&self) -> Vec<String> {
        vec!["npx".into(), "prisma".into(), "migrate".into(), "dev".into()]
    }

    fn get_seed_command(&self) -> Vec<String> {
        vec!["npx".into(), "prisma".into(), "db".into(), "seed".into()]
    }

    fn validate_schema(&self, project_dir: &Path) -> Result<Vec<String>, OrmError> {
        let schema_path = project_dir.join("prisma/schema.prisma");
        if !schema_path.exists() {
            return Err(OrmError::NotFound("prisma/schema.prisma not found".into()));
        }
        let content = std::fs::read_to_string(&schema_path)
            .map_err(|e| OrmError::SchemaError(format!("Cannot read schema: {e}")))?;
        let mut issues = Vec::new();
        if !content.contains("datasource") {
            issues.push("Missing datasource block in schema.prisma".into());
        }
        if !content.contains("generator") {
            issues.push("Missing generator block in schema.prisma".into());
        }
        if !content.contains("model ") && !content.contains("enum ") {
            issues.push("No models or enums defined in schema.prisma".into());
        }
        Ok(issues)
    }

    fn get_supported_databases(&self) -> Vec<&'static str> {
        vec!["postgresql", "mysql", "sqlite", "sqlserver", "mongodb", "cockroachdb"]
    }
}
