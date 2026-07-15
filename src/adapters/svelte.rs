use std::collections::HashMap;
use std::path::Path;
use crate::adapters::{AdapterTrait, AdapterConfig, AdapterError, AdapterDep, TemplateFile};

pub struct SvelteAdapter;

fn has_dep(pkg: &str, content: &str) -> bool {
    content.contains(&format!("\"{pkg}\""))
}

fn read_package_json(dir: &Path) -> Option<String> {
    let path = dir.join("package.json");
    if path.exists() { std::fs::read_to_string(path).ok() } else { None }
}

impl AdapterTrait for SvelteAdapter {
    fn name(&self) -> &'static str { "svelte" }

    fn detect(&self, dir: &Path) -> bool {
        if dir.join("svelte.config.js").exists() || dir.join("svelte.config.ts").exists() {
            return true;
        }
        if let Some(content) = read_package_json(dir) {
            let has_svelte = has_dep("svelte", &content);
            let has_kit = has_dep("@sveltejs/kit", &content);
            let has_vite_svelte = has_dep("@sveltejs/vite-plugin-svelte", &content);
            if has_svelte && (has_kit || has_vite_svelte) {
                return true;
            }
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["svelte"].as_str().or_else(|| pkg["devDependencies"]["svelte"].as_str()).map(|s| s.to_string());
        let is_kit = has_dep("@sveltejs/kit", &content);
        let build_cmd = if is_kit { Some("svelte-kit build".into()) } else { Some("vite build".into()) };
        let dev_cmd = if is_kit { Some("svelte-kit dev".into()) } else { Some("vite".into()) };
        let out_dir = if is_kit { Some(".svelte-kit".into()) } else { Some("dist".into()) };
        Ok(AdapterConfig {
            name: "svelte".into(),
            version,
            build_command: build_cmd,
            dev_command: dev_cmd,
            output_dir: out_dir,
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
            AdapterDep { name: "svelte".into(), version: "^5.2.0".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "@sveltejs/vite-plugin-svelte".into(), version: "^5.0.0".into(), is_dev: true, is_optional: false },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/svelte/App.svelte".into(), dest: "src/App.svelte".into(), is_template: true },
            TemplateFile { source: "templates/svelte/main.ts".into(), dest: "src/main.ts".into(), is_template: false },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec!["src/hooks.server.ts".into(), "src/hooks.client.ts".into()]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["src/routes/**/*.svelte".into(), "src/routes/**/+page.svelte".into(), "src/routes/**/+server.ts".into()]
    }
}
