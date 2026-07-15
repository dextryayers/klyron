use std::collections::HashMap;
use std::path::Path;
use crate::adapters::{AdapterTrait, AdapterConfig, AdapterError, AdapterDep, TemplateFile};

pub struct HonoAdapter;

fn has_dep(pkg: &str, content: &str) -> bool {
    content.contains(&format!("\"{pkg}\""))
}

fn read_package_json(dir: &Path) -> Option<String> {
    let path = dir.join("package.json");
    if path.exists() { std::fs::read_to_string(path).ok() } else { None }
}

impl AdapterTrait for HonoAdapter {
    fn name(&self) -> &'static str { "hono" }

    fn detect(&self, dir: &Path) -> bool {
        if let Some(content) = read_package_json(dir) {
            return has_dep("hono", &content);
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["hono"].as_str().map(|s| s.to_string());
        Ok(AdapterConfig {
            name: "hono".into(),
            version,
            build_command: None,
            dev_command: Some("tsx watch src/index.ts".into()),
            output_dir: Some("dist".into()),
            src_dir: Some("src".into()),
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
        if !dir.join("src/index.ts").exists() && !dir.join("src/index.js").exists() {
            issues.push("Missing src/index.ts or src/index.js".into());
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
