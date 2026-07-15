use std::collections::HashMap;
use std::path::Path;
use crate::adapters::{AdapterTrait, AdapterConfig, AdapterError, AdapterDep, TemplateFile};

pub struct NuxtAdapter;

fn has_dep(pkg: &str, content: &str) -> bool {
    content.contains(&format!("\"{pkg}\""))
}

fn read_package_json(dir: &Path) -> Option<String> {
    let path = dir.join("package.json");
    if path.exists() { std::fs::read_to_string(path).ok() } else { None }
}

impl AdapterTrait for NuxtAdapter {
    fn name(&self) -> &'static str { "nuxt" }

    fn detect(&self, dir: &Path) -> bool {
        if dir.join("nuxt.config.ts").exists() || dir.join("nuxt.config.js").exists() || dir.join("nuxt.config.mjs").exists() {
            return true;
        }
        if let Some(content) = read_package_json(dir) {
            return has_dep("nuxt", &content) || has_dep("nuxt3", &content) || has_dep("nuxt-edge", &content);
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["nuxt"].as_str().or_else(|| pkg["devDependencies"]["nuxt"].as_str()).map(|s| s.to_string());
        Ok(AdapterConfig {
            name: "nuxt".into(),
            version,
            build_command: Some("nuxt build".into()),
            dev_command: Some("nuxt dev".into()),
            output_dir: Some(".output".into()),
            src_dir: Some(".".into()),
            port: Some(3000),
            node_version: None,
            custom: HashMap::new(),
        })
    }

    fn get_build_command(&self) -> Vec<String> {
        vec!["npx".into(), "nuxt".into(), "build".into()]
    }

    fn get_dev_command(&self) -> Vec<String> {
        vec!["npx".into(), "nuxt".into(), "dev".into(), "--port".into(), "3000".into()]
    }

    fn get_output_dir(&self) -> &'static str {
        ".output"
    }

    fn validate_project(&self, dir: &Path) -> Result<Vec<String>, AdapterError> {
        let mut issues = Vec::new();
        if !dir.join("package.json").exists() {
            issues.push("Missing package.json".into());
        }
        if !dir.join("nuxt.config.ts").exists() && !dir.join("nuxt.config.js").exists() {
            issues.push("Missing nuxt.config.ts or nuxt.config.js".into());
        }
        Ok(issues)
    }

    fn get_dependencies(&self) -> Vec<AdapterDep> {
        vec![
            AdapterDep { name: "nuxt".into(), version: "^3.15.0".into(), is_dev: false, is_optional: false },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/nuxt/app.vue".into(), dest: "app.vue".into(), is_template: true },
            TemplateFile { source: "templates/nuxt/nuxt.config.ts".into(), dest: "nuxt.config.ts".into(), is_template: false },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec!["middleware/**/*.ts".into(), "server/middleware/**/*.ts".into()]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["pages/**/*.vue".into(), "pages/**/*.ts".into()]
    }
}
