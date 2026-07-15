use std::collections::HashMap;
use std::path::Path;
use crate::adapters::{AdapterTrait, AdapterConfig, AdapterError, AdapterDep, TemplateFile};

pub struct ReactAdapter;

fn has_dep(pkg: &str, content: &str) -> bool {
    content.contains(&format!("\"{pkg}\""))
}

fn read_package_json(dir: &Path) -> Option<String> {
    let path = dir.join("package.json");
    if path.exists() { std::fs::read_to_string(path).ok() } else { None }
}

impl AdapterTrait for ReactAdapter {
    fn name(&self) -> &'static str { "react" }

    fn detect(&self, dir: &Path) -> bool {
        if let Some(content) = read_package_json(dir) {
            let has_react = has_dep("react", &content);
            let has_cra = has_dep("react-scripts", &content);
            let has_vite_react = has_dep("@vitejs/plugin-react", &content);
            let has_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
            if has_react && (has_cra || has_vite_react || has_vite) {
                return true;
            }
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["react"].as_str().map(|s| s.to_string());
        let is_vite = dir.join("vite.config.ts").exists() || dir.join("vite.config.js").exists();
        let is_cra = has_dep("react-scripts", &content);
        let build_cmd = if is_vite { Some("vite build".into()) } else if is_cra { Some("react-scripts build".into()) } else { None };
        let dev_cmd = if is_vite { Some("vite".into()) } else if is_cra { Some("react-scripts start".into()) } else { None };
        let out_dir = if is_vite { Some("dist".into()) } else if is_cra { Some("build".into()) } else { None };
        Ok(AdapterConfig {
            name: "react".into(),
            version,
            build_command: build_cmd,
            dev_command: dev_cmd,
            output_dir: out_dir,
            src_dir: Some("src".into()),
            port: Some(3000),
            node_version: None,
            custom: HashMap::new(),
        })
    }

    fn get_build_command(&self) -> Vec<String> {
        vec!["npx".into(), "vite".into(), "build".into()]
    }

    fn get_dev_command(&self) -> Vec<String> {
        vec!["npx".into(), "vite".into(), "--port".into(), "3000".into()]
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
            AdapterDep { name: "react".into(), version: "^19.1.0".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "react-dom".into(), version: "^19.1.0".into(), is_dev: false, is_optional: false },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/react/App.tsx".into(), dest: "src/App.tsx".into(), is_template: true },
            TemplateFile { source: "templates/react/main.tsx".into(), dest: "src/main.tsx".into(), is_template: false },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec![]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["src/pages/**/*.tsx".into(), "src/pages/**/*.jsx".into()]
    }
}
