use std::collections::HashMap;
use std::path::Path;
use crate::adapters::{AdapterTrait, AdapterConfig, AdapterError, AdapterDep, TemplateFile};

fn has_dep(pkg: &str, content: &str) -> bool {
    content.contains(&format!("\"{pkg}\""))
}

fn read_package_json(dir: &Path) -> Option<String> {
    let path = dir.join("package.json");
    if path.exists() { std::fs::read_to_string(path).ok() } else { None }
}

pub struct ElysiaAdapter;

impl AdapterTrait for ElysiaAdapter {
    fn name(&self) -> &'static str { "elysia" }

    fn detect(&self, dir: &Path) -> bool {
        if let Some(content) = read_package_json(dir) {
            return has_dep("elysia", &content);
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["elysia"].as_str().map(|s| s.to_string());
        let src_file = dir.join("src/index.ts");
        let port = if src_file.exists() {
            std::fs::read_to_string(&src_file).ok().and_then(|s| {
                s.lines().find_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.contains(".listen(") {
                        trimmed.split(".listen(").nth(1).and_then(|part| {
                            part.trim_end_matches(')').trim().split(',').next().and_then(|n| {
                                n.trim().trim_matches(&['{', ' ', '}'][..]).parse::<u16>().ok()
                            })
                        })
                    } else {
                        None
                    }
                })
            })
        } else {
            None
        };
        Ok(AdapterConfig {
            name: "elysia".into(),
            version,
            build_command: Some("bun build ./src/index.ts".into()),
            dev_command: Some("bun --watch src/index.ts".into()),
            output_dir: Some("dist".into()),
            src_dir: Some("src".into()),
            port: port.or(Some(3000)),
            node_version: None,
            custom: HashMap::new(),
        })
    }

    fn get_build_command(&self) -> Vec<String> {
        vec!["npx".into(), "bun".into(), "build".into(), "./src/index.ts".into()]
    }

    fn get_dev_command(&self) -> Vec<String> {
        vec!["npx".into(), "bun".into(), "--watch".into(), "src/index.ts".into()]
    }

    fn get_output_dir(&self) -> &'static str {
        "dist"
    }

    fn validate_project(&self, dir: &Path) -> Result<Vec<String>, AdapterError> {
        let mut issues = Vec::new();
        if !dir.join("package.json").exists() {
            issues.push("Missing package.json".into());
        }
        if !dir.join("src/index.ts").exists() {
            issues.push("Missing src/index.ts".into());
        }
        Ok(issues)
    }

    fn get_dependencies(&self) -> Vec<AdapterDep> {
        vec![
            AdapterDep { name: "elysia".into(), version: "latest".into(), is_dev: false, is_optional: false },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/express/index.js".into(), dest: "src/index.ts".into(), is_template: true },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec!["src/middleware/*.ts".into()]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["src/routes/*.ts".into(), "src/index.ts".into()]
    }
}
