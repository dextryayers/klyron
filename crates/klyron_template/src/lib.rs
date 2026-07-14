use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: Option<String>,
    pub kind: TemplateKind,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemplateKind {
    Frontend,
    Backend,
    Fullstack,
    Library,
    Component,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVar {
    pub name: String,
    pub var_type: VarType,
    pub prompt: Option<String>,
    pub default: Option<String>,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VarType {
    String,
    Number,
    Select,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    pub vars: HashMap<String, String>,
    pub project_name: String,
    pub project_dir: PathBuf,
}

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn render(content: &str, vars: &HashMap<String, String>) -> String {
        let mut result = content.to_string();
        for (key, value) in vars {
            let pattern_simple = format!("{{{{ {} }}}}", key);
            result = result.replace(&pattern_simple, value.as_str());

            let pattern_nospace = format!("{{{{{}}}}}", key);
            result = result.replace(&pattern_nospace, value.as_str());

            let pattern_upper = format!("{{{{ {} | upper }}}}", key);
            result = result.replace(&pattern_upper, &value.to_uppercase());

            let pattern_lower = format!("{{{{ {} | lower }}}}", key);
            result = result.replace(&pattern_lower, &value.to_lowercase());

            let pattern_cap = format!("{{{{ {} | capitalize }}}}", key);
            let capitalized = value[..1].to_uppercase() + &value[1..];
            result = result.replace(&pattern_cap, &capitalized);
        }
        result
    }

    pub fn scaffold(template_dir: &Path, dest: &Path, ctx: &TemplateContext) -> anyhow::Result<()> {
        let files_dir = template_dir.join("files");
        if !files_dir.exists() {
            anyhow::bail!("Template files directory not found: {}", files_dir.display());
        }
        Self::copy_dir(&files_dir, dest, ctx)?;

        let hooks_dir = template_dir.join("hooks");
        let post_hook = hooks_dir.join("post-scaffold.sh");
        if post_hook.exists() {
            let script = std::fs::read_to_string(&post_hook)?;
            let rendered = Self::render(&script, &ctx.vars);
            let temp_script = dest.join(".post-scaffold.sh");
            std::fs::write(&temp_script, rendered)?;
            #[cfg(unix)]
            std::fs::set_permissions(&temp_script, std::os::unix::fs::PermissionsExt::from_mode(0o755))?;
        }
        Ok(())
    }

    fn copy_dir(src: &Path, dest: &Path, ctx: &TemplateContext) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let relative = src_path.strip_prefix(src).unwrap();
            let rendered_name = Self::render(&relative.to_string_lossy(), &ctx.vars);

            let dest_path = if rendered_name.contains("{{") {
                dest.join(Self::render(&relative.to_string_lossy(), &ctx.vars))
            } else {
                dest.join(relative)
            };

            if file_type.is_dir() {
                std::fs::create_dir_all(&dest_path)?;
                Self::copy_dir(&src_path, &dest_path, ctx)?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let content = std::fs::read_to_string(&src_path).unwrap_or_default();
                let rendered = Self::render(&content, &ctx.vars);
                std::fs::write(&dest_path, rendered.as_bytes())?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "my-app".to_string());
        let result = TemplateEngine::render("Project: {{ name }}", &vars);
        assert_eq!(result, "Project: my-app");
    }

    #[test]
    fn test_render_upper() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "my-app".to_string());
        let result = TemplateEngine::render("Project: {{ name | upper }}", &vars);
        assert_eq!(result, "Project: MY-APP");
    }

    #[test]
    fn test_render_capitalize() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "hello".to_string());
        let result = TemplateEngine::render("{{ name | capitalize }}", &vars);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_render_lower() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "HELLO".to_string());
        let result = TemplateEngine::render("{{ name | lower }}", &vars);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_render_multiple_vars() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "app".to_string());
        vars.insert("version".to_string(), "1.0.0".to_string());
        let result = TemplateEngine::render(r#"{"name": "{{ name }}", "version": "{{ version }}"}"#, &vars);
        assert_eq!(result, r#"{"name": "app", "version": "1.0.0"}"#);
    }

    #[test]
    fn test_render_nospace() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "myapp".to_string());
        let result = TemplateEngine::render("const name = '{{name}}';", &vars);
        assert_eq!(result, "const name = 'myapp';");
    }

    #[test]
    fn test_scaffold_with_template() {
        let tmp = std::env::temp_dir().join(format!("klyron_tpl_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let template_dir = tmp.join("template");
        let files_dir = template_dir.join("files");
        std::fs::create_dir_all(&files_dir).unwrap();
        std::fs::write(files_dir.join("greeting.txt"), "Hello, {{ name }}!").unwrap();

        let dest = tmp.join("output");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        let ctx = TemplateContext {
            vars,
            project_name: "test".to_string(),
            project_dir: dest.clone(),
        };

        TemplateEngine::scaffold(&template_dir, &dest, &ctx).unwrap();
        let output = std::fs::read_to_string(dest.join("greeting.txt")).unwrap();
        assert_eq!(output, "Hello, World!");
    }

    #[test]
    fn test_scaffold_with_nested_dirs() {
        let tmp = std::env::temp_dir().join(format!("klyron_nest_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let template_dir = tmp.join("template");
        std::fs::create_dir_all(template_dir.join("files/src/pages")).unwrap();
        std::fs::write(template_dir.join("files/src/pages/index.ts"), "export default {{ name | capitalize }}").unwrap();
        std::fs::write(template_dir.join("files/package.json"), r#"{"name": "{{ name }}"}"#).unwrap();

        let dest = tmp.join("output");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "myapp".to_string());
        let ctx = TemplateContext {
            vars,
            project_name: "myapp".to_string(),
            project_dir: dest.clone(),
        };

        TemplateEngine::scaffold(&template_dir, &dest, &ctx).unwrap();
        let pkg = std::fs::read_to_string(dest.join("package.json")).unwrap();
        assert_eq!(pkg, r#"{"name": "myapp"}"#);
        let page = std::fs::read_to_string(dest.join("src/pages/index.ts")).unwrap();
        assert_eq!(page, "export default Myapp");
    }
}
