use std::collections::HashMap;
use std::path::Path;
use crate::adapters::{AdapterTrait, AdapterConfig, AdapterError, AdapterDep, TemplateFile};

pub struct NextAdapter;

fn has_dep(pkg: &str, content: &str) -> bool {
    content.contains(&format!("\"{pkg}\""))
}

fn read_package_json(dir: &Path) -> Option<String> {
    let path = dir.join("package.json");
    if path.exists() {
        std::fs::read_to_string(path).ok()
    } else {
        None
    }
}

impl AdapterTrait for NextAdapter {
    fn name(&self) -> &'static str { "next" }

    fn detect(&self, dir: &Path) -> bool {
        if dir.join("next.config.ts").exists() || dir.join("next.config.js").exists() || dir.join("next.config.mjs").exists() {
            return true;
        }
        if let Some(content) = read_package_json(dir) {
            return has_dep("next", &content);
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["next"].as_str().or_else(|| pkg["devDependencies"]["next"].as_str()).map(|s| s.to_string());
        let src_dir = if dir.join("app").exists() { Some("app".into()) } else if dir.join("pages").exists() { Some("pages".into()) } else { Some(".".into()) };
        Ok(AdapterConfig {
            name: "next".into(),
            version,
            build_command: Some("next build".into()),
            dev_command: Some("next dev".into()),
            output_dir: Some(".next".into()),
            src_dir,
            port: Some(3000),
            node_version: None,
            custom: HashMap::new(),
        })
    }

    fn get_build_command(&self) -> Vec<String> {
        vec!["npx".into(), "next".into(), "build".into()]
    }

    fn get_dev_command(&self) -> Vec<String> {
        vec!["npx".into(), "next".into(), "dev".into(), "-p".into(), "3000".into()]
    }

    fn get_output_dir(&self) -> &'static str {
        ".next"
    }

    fn validate_project(&self, dir: &Path) -> Result<Vec<String>, AdapterError> {
        let mut issues = Vec::new();
        if !dir.join("package.json").exists() {
            issues.push("Missing package.json".into());
        }
        if !dir.join("next.config.js").exists() && !dir.join("next.config.ts").exists() && !dir.join("next.config.mjs").exists() {
            issues.push("Missing next.config.js/ts/mjs".into());
        }
        if !dir.join("pages").exists() && !dir.join("app").exists() {
            issues.push("Missing pages/ or app/ directory".into());
        }
        Ok(issues)
    }

    fn get_dependencies(&self) -> Vec<AdapterDep> {
        vec![
            AdapterDep { name: "next".into(), version: "^15.3.0".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "react".into(), version: "^19.1.0".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "react-dom".into(), version: "^19.1.0".into(), is_dev: false, is_optional: false },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/next/layout.tsx".into(), dest: "app/layout.tsx".into(), is_template: true },
            TemplateFile { source: "templates/next/page.tsx".into(), dest: "app/page.tsx".into(), is_template: true },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec!["src/middleware.ts".into(), "src/middleware/index.ts".into(), "middleware.ts".into()]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["app/**/page.tsx".into(), "app/**/route.ts".into(), "pages/**/*.tsx".into(), "pages/**/*.js".into()]
    }
}
