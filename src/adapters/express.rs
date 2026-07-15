use std::collections::HashMap;
use std::path::Path;
use crate::adapters::{AdapterTrait, AdapterConfig, AdapterError, AdapterDep, TemplateFile};

pub struct ExpressAdapter;

fn has_dep(pkg: &str, content: &str) -> bool {
    content.contains(&format!("\"{pkg}\""))
}

fn read_package_json(dir: &Path) -> Option<String> {
    let path = dir.join("package.json");
    if path.exists() { std::fs::read_to_string(path).ok() } else { None }
}

impl AdapterTrait for ExpressAdapter {
    fn name(&self) -> &'static str { "express" }

    fn detect(&self, dir: &Path) -> bool {
        if let Some(content) = read_package_json(dir) {
            return has_dep("express", &content);
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["express"].as_str().map(|s| s.to_string());
        let src_dir = if dir.join("src").exists() { Some("src".into()) } else { Some(".".into()) };
        Ok(AdapterConfig {
            name: "express".into(),
            version,
            build_command: None,
            dev_command: Some("node --watch src/index.js".into()),
            output_dir: None,
            src_dir,
            port: Some(3000),
            node_version: None,
            custom: HashMap::new(),
        })
    }

    fn get_build_command(&self) -> Vec<String> {
        vec!["npx".into(), "tsc".into()]
    }

    fn get_dev_command(&self) -> Vec<String> {
        vec!["npx".into(), "tsx".into(), "watch".into(), "src/index.ts".into()]
    }

    fn get_output_dir(&self) -> &'static str {
        "dist"
    }

    fn validate_project(&self, dir: &Path) -> Result<Vec<String>, AdapterError> {
        let mut issues = Vec::new();
        if !dir.join("package.json").exists() {
            issues.push("Missing package.json".into());
        }
        if !dir.join("src").exists() || (!dir.join("src/index.js").exists() && !dir.join("src/index.ts").exists()) {
            issues.push("Missing src/index.js or src/index.ts".into());
        }
        Ok(issues)
    }

    fn get_dependencies(&self) -> Vec<AdapterDep> {
        vec![
            AdapterDep { name: "express".into(), version: "^5.1.0".into(), is_dev: false, is_optional: false },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/express/index.js".into(), dest: "src/index.js".into(), is_template: true },
            TemplateFile { source: "templates/express/routes.js".into(), dest: "src/routes/index.js".into(), is_template: false },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec!["src/middleware/*.js".into(), "src/middleware/*.ts".into(), "middleware/*.js".into()]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["src/routes/**/*.js".into(), "src/routes/**/*.ts".into(), "routes/**/*.js".into()]
    }
}
