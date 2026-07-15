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

pub struct NestAdapter;

impl AdapterTrait for NestAdapter {
    fn name(&self) -> &'static str { "nestjs" }

    fn detect(&self, dir: &Path) -> bool {
        if let Some(content) = read_package_json(dir) {
            return has_dep("@nestjs/core", &content);
        }
        false
    }

    fn get_config(&self, dir: &Path) -> Result<AdapterConfig, AdapterError> {
        let content = read_package_json(dir).ok_or_else(|| AdapterError::MissingFile("package.json".into()))?;
        let pkg: serde_json::Value = serde_json::from_str(&content).map_err(|e| AdapterError::InvalidConfig(e.to_string()))?;
        let version = pkg["dependencies"]["@nestjs/core"].as_str().map(|s| s.to_string());
        let src_dir = Some("src".into());
        let port = {
            let main_path = dir.join("src/main.ts");
            main_path.exists().then(|| {
                std::fs::read_to_string(&main_path).ok().and_then(|s| {
                    s.lines().find_map(|line| {
                        let trimmed = line.trim();
                        if trimmed.contains("await") && trimmed.contains("listen") {
                            trimmed.split('(').nth(1).and_then(|part| {
                                part.trim_end_matches(&[')', ';'][..]).split(',').next().and_then(|n| {
                                    n.trim().trim_matches('"').trim().parse::<u16>().ok()
                                })
                            })
                        } else {
                            None
                        }
                    })
                })
            }).flatten()
        };
        Ok(AdapterConfig {
            name: "nestjs".into(),
            version,
            build_command: Some("nest build".into()),
            dev_command: Some("nest start --watch".into()),
            output_dir: Some("dist".into()),
            src_dir,
            port: port.or(Some(3000)),
            node_version: None,
            custom: HashMap::new(),
        })
    }

    fn get_build_command(&self) -> Vec<String> {
        vec!["npx".into(), "nest".into(), "build".into()]
    }

    fn get_dev_command(&self) -> Vec<String> {
        vec!["npx".into(), "nest".into(), "start".into(), "--watch".into()]
    }

    fn get_output_dir(&self) -> &'static str {
        "dist"
    }

    fn validate_project(&self, dir: &Path) -> Result<Vec<String>, AdapterError> {
        let mut issues = Vec::new();
        if !dir.join("package.json").exists() {
            issues.push("Missing package.json".into());
        }
        if !dir.join("src/main.ts").exists() {
            issues.push("Missing src/main.ts".into());
        }
        if !dir.join("nest-cli.json").exists() {
            issues.push("Missing nest-cli.json".into());
        }
        if !dir.join("tsconfig.json").exists() {
            issues.push("Missing tsconfig.json".into());
        }
        if !dir.join("tsconfig.build.json").exists() {
            issues.push("Missing tsconfig.build.json".into());
        }
        Ok(issues)
    }

    fn get_dependencies(&self) -> Vec<AdapterDep> {
        vec![
            AdapterDep { name: "@nestjs/core".into(), version: "^11.0.0".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "@nestjs/common".into(), version: "^11.0.0".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "@nestjs/platform-express".into(), version: "^11.0.0".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "reflect-metadata".into(), version: "^0.2.2".into(), is_dev: false, is_optional: false },
            AdapterDep { name: "typescript".into(), version: "^5.8.0".into(), is_dev: true, is_optional: false },
        ]
    }

    fn get_template_files(&self) -> Vec<TemplateFile> {
        vec![
            TemplateFile { source: "templates/express/index.js".into(), dest: "src/main.ts".into(), is_template: true },
            TemplateFile { source: "templates/express/index.js".into(), dest: "src/app.module.ts".into(), is_template: true },
            TemplateFile { source: "templates/express/index.js".into(), dest: "src/app.controller.ts".into(), is_template: true },
        ]
    }

    fn get_middleware_pattern(&self) -> Vec<String> {
        vec!["src/common/middleware/*.ts".into()]
    }

    fn get_route_pattern(&self) -> Vec<String> {
        vec!["src/**/*.controller.ts".into()]
    }
}
