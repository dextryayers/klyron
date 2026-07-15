use std::collections::HashMap;
use std::path::Path;
use crate::adapters::{AdapterTrait, AdapterConfig, AdapterError, AdapterDep, TemplateFile};

pub struct AstroAdapter;

fn has_dep(pkg: &str, content: &str) -> bool {
    content.contains(&format!("\"{pkg}\""))
}

fn read_package_json(dir: &Path) -> Option<String> {
    let path = dir.join("package.json");
    if path.exists() { std::fs::read_to_string(path).ok() } else { None }
}

impl AdapterTrait for AstroAdapter {
    fn name(&self) -> &'static str { "astro" }

    fn detect(&self, dir: &Path) -> bool {
        if dir.join("astro.config.mjs").exists() || dir.join("astro.config.ts").exists() || dir.join("astro.config.js").exists() {
            return true;
        }
        if let Some(content) = read_package_json(dir) {
            return has_dep("astro", &content);
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["astro"].as_str().or_else(|| pkg["devDependencies"]["astro"].as_str()).map(|s| s.to_string());
        Ok(AdapterConfig {
            name: "astro".into(),
            version,
            build_command: Some("astro build".into()),
            dev_command: Some("astro dev".into()),
            output_dir: Some("dist".into()),
            src_dir: Some("src".into()),
            port: Some(4321),
            node_version: None,
            custom: HashMap::new(),
        })
    }

    fn get_build_command(&self) -> Vec<String> {
        vec!["npx".into(), "astro".into(), "build".into()]
    }

    fn get_dev_command(&self) -> Vec<String> {
        vec!["npx".into(), "astro".into(), "dev".into(), "--port".into(), "4321".into()]
    }

    fn get_output_dir(&self) -> &'static str {
        "dist"
    }

    fn validate_project(&self, dir: &Path) -> Result<Vec<String>, AdapterError> {
        let mut issues = Vec::new();
        if !dir.join("package.json").exists() {
            issues.push("Missing package.json".into());
        }
        if !dir.join("astro.config.mjs").exists() && !dir.join("astro.config.ts").exists() && !dir.join("astro.config.js").exists() {
            issues.push("Missing astro.config.mjs/ts/js".into());
        }
        if !dir.join("src/pages").exists() {
            issues.push("Missing src/pages directory".into());
        }
        Ok(issues)
    }

    fn get_dependencies(&self) -> Vec<AdapterDep> {
        vec![
            AdapterDep { name: "astro".into(), version: "^5.4.0".into(), is_dev: false, is_optional: false },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/astro/index.astro".into(), dest: "src/pages/index.astro".into(), is_template: true },
            TemplateFile { source: "templates/astro/layout.astro".into(), dest: "src/layouts/Layout.astro".into(), is_template: false },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec!["src/middleware.ts".into(), "src/middleware/index.ts".into()]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["src/pages/**/*.astro".into(), "src/pages/**/*.md".into(), "src/pages/**/*.mdx".into()]
    }
}
