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

pub struct FastifyAdapter;

impl AdapterTrait for FastifyAdapter {
    fn name(&self) -> &'static str { "fastify" }

    fn detect(&self, dir: &Path) -> bool {
        if let Some(content) = read_package_json(dir) {
            return has_dep("fastify", &content);
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["fastify"].as_str().map(|s| s.to_string());
        let src_dir = if dir.join("src").exists() { Some("src".into()) } else { Some(".".into()) };
        let port = {
            let candidates = ["src/app.ts", "app.ts", "src/index.ts", "index.ts"];
            candidates.iter().find_map(|p| {
                let path = dir.join(p);
                path.exists().then(|| {
                    std::fs::read_to_string(&path).ok().and_then(|s| {
                        s.lines().find_map(|line| {
                            let trimmed = line.trim();
                            if trimmed.contains(".listen(") {
                                trimmed.split(".listen(").nth(1).and_then(|part| {
                                    part.split(&[')', ','][..]).next().and_then(|n| {
                                        n.trim().parse::<u16>().ok()
                                    })
                                })
                            } else {
                                None
                            }
                        })
                    })
                }).flatten()
            })
        };
        Ok(AdapterConfig {
            name: "fastify".into(),
            version,
            build_command: Some("tsc".into()),
            dev_command: Some("tsx watch src/app.ts".into()),
            output_dir: Some("dist".into()),
            src_dir,
            port: port.or(Some(3000)),
            node_version: None,
            custom: HashMap::new(),
        })
    }

    fn get_build_command(&self) -> Vec<String> {
        vec!["npx".into(), "tsc".into()]
    }

    fn get_dev_command(&self) -> Vec<String> {
        vec!["npx".into(), "tsx".into(), "watch".into(), "src/app.ts".into()]
    }

    fn get_output_dir(&self) -> &'static str {
        "dist"
    }

    fn validate_project(&self, dir: &Path) -> Result<Vec<String>, AdapterError> {
        let mut issues = Vec::new();
        if !dir.join("package.json").exists() {
            issues.push("Missing package.json".into());
        }
        if !dir.join("src/app.ts").exists() && !dir.join("app.ts").exists() {
            issues.push("Missing src/app.ts or app.ts".into());
        }
        Ok(issues)
    }

    fn get_dependencies(&self) -> Vec<AdapterDep> {
        vec![
            AdapterDep { name: "fastify".into(), version: "^5.3.0".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "@fastify/cors".into(), version: "^11.0.0".into(), is_dev: false, is_optional: true },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/express/index.js".into(), dest: "src/app.ts".into(), is_template: true },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec!["src/middleware/*.ts".into(), "src/plugins/*.ts".into()]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["src/routes/**/*.ts".into(), "src/app.ts".into()]
    }
}
