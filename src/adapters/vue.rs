use std::collections::HashMap;
use std::path::Path;
use crate::adapters::{AdapterTrait, AdapterConfig, AdapterError, AdapterDep, TemplateFile};

pub struct VueAdapter;

fn has_dep(pkg: &str, content: &str) -> bool {
    content.contains(&format!("\"{pkg}\""))
}

fn read_package_json(dir: &Path) -> Option<String> {
    let path = dir.join("package.json");
    if path.exists() { std::fs::read_to_string(path).ok() } else { None }
}

impl AdapterTrait for VueAdapter {
    fn name(&self) -> &'static str { "vue" }

    fn detect(&self, dir: &Path) -> bool {
        if let Some(content) = read_package_json(dir) {
            let has_vue = has_dep("vue", &content);
            let has_vue_cli = has_dep("@vue/cli-service", &content);
            let has_vite_vue = has_dep("@vitejs/plugin-vue", &content);
            let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
            if has_vue && (has_vue_cli || has_vite_vue || has_vite) {
                return true;
            }
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["vue"].as_str().map(|s| s.to_string());
        let is_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
        let build_cmd = if is_vite { Some("vite build".into()) } else { Some("vue-cli-service build".into()) };
        let dev_cmd = if is_vite { Some("vite".into()) } else { Some("vue-cli-service serve".into()) };
        Ok(AdapterConfig {
            name: "vue".into(),
            version,
            build_command: build_cmd,
            dev_command: dev_cmd,
            output_dir: Some("dist".into()),
            src_dir: Some("src".into()),
            port: Some(5173),
            node_version: None,
            custom: HashMap::new(),
        })
    }

    fn get_build_command(&self) -> Vec<String> {
        vec!["npx".into(), "vite".into(), "build".into()]
    }

    fn get_dev_command(&self) -> Vec<String> {
        vec!["npx".into(), "vite".into(), "--port".into(), "5173".into()]
    }

    fn get_output_dir(&self) -> &'static str {
        "dist"
    }

    fn validate_project(&self, dir: &Path) -> Result<Vec<String>, AdapterError> {
        let mut issues = Vec::new();
        if !dir.join("package.json").exists() {
            issues.push("Missing package.json".into());
        }
        if !dir.join("vite.config.ts").exists() && !dir.join("vite.config.js").exists() {
            issues.push("Missing vite.config.ts or vite.config.js".into());
        }
        if !dir.join("index.html").exists() {
            issues.push("Missing index.html".into());
        }
        Ok(issues)
    }

    fn get_dependencies(&self) -> Vec<AdapterDep> {
        vec![
            AdapterDep { name: "vue".into(), version: "^3.5.0".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "@vitejs/plugin-vue".into(), version: "^5.2.0".into(), is_dev: true, is_optional: false },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/vue/App.vue".into(), dest: "src/App.vue".into(), is_template: true },
            TemplateFile { source: "templates/vue/main.ts".into(), dest: "src/main.ts".into(), is_template: false },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec![]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["src/router/**/*.ts".into(), "src/pages/**/*.vue".into()]
    }
}
