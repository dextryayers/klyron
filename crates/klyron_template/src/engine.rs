use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use regex::Regex;
use walkdir::WalkDir;

use crate::helpers::TemplateHelpers;

#[derive(Clone)]
pub struct TemplateEngine {
    filters: HashMap<String, Arc<dyn Fn(&str) -> String + Send + Sync>>,
    partials: HashMap<String, String>,
    blocks: HashMap<String, String>,
    helpers: TemplateHelpers,
}

impl std::fmt::Debug for TemplateEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateEngine")
            .field("partials", &self.partials)
            .field("blocks", &self.blocks)
            .finish()
    }
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            filters: HashMap::new(),
            partials: HashMap::new(),
            blocks: HashMap::new(),
            helpers: TemplateHelpers::new(),
        };
        engine.register_default_filters();
        engine
    }

    fn register_default_filters(&mut self) {
        self.register_filter("upper", |s| s.to_uppercase());
        self.register_filter("lower", |s| s.to_lowercase());
        self.register_filter("capitalize", |s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        });
        self.register_filter("trim", |s| s.trim().to_string());
        self.register_filter("reverse", |s| s.chars().rev().collect());
        self.register_filter("length", |s| s.len().to_string());
        self.register_filter("default", |s| if s.is_empty() { "default".into() } else { s.into() });
        self.register_filter("json", |s| serde_json::to_string(s).unwrap_or_else(|_| s.to_string()));
    }

    pub fn register_filter<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.filters.insert(name.to_string(), Arc::new(f));
    }

    pub fn add_partial(&mut self, name: &str, content: &str) {
        self.partials.insert(name.to_string(), content.to_string());
    }

    pub fn load_partials_dir(&mut self, dir: &Path) -> anyhow::Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    if ext == "html" || ext == "njk" || ext == "hbs" {
                        let name = entry.path()
                            .strip_prefix(dir)
                            .unwrap_or(entry.path())
                            .with_extension("")
                            .to_string_lossy()
                            .to_string()
                            .replace(std::path::MAIN_SEPARATOR, "/");
                        let content = std::fs::read_to_string(entry.path())?;
                        self.add_partial(&name, &content);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn render_static(template: &str, vars: &HashMap<String, String>) -> String {
        Self::new().render(template, vars)
    }

    pub fn render(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        result = self.render_nunjucks(&result, vars);
        result = self.render_handlebars(&result, vars);
        result = self.render_variables(&result, vars);
        result
    }

    pub fn render_with_inheritance(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let extends_re = Regex::new(r#"\{%\s*extends\s+"([^"]+)"\s*%\}"#).unwrap();
        let block_re = Regex::new(r"\{%\s*block\s+(\w+)\s*%\}(.*?)\{%\s*endblock\s*%\}")
            .unwrap_or_else(|_| Regex::new(r"").unwrap());

        let mut result = template.to_string();

        if let Some(caps) = extends_re.captures(&result) {
            let layout_name = caps.get(1).unwrap().as_str();
            if let Some(layout) = self.partials.get(layout_name) {
                result = layout.to_string();
                for block_cap in block_re.captures_iter(template) {
                    let block_name = block_cap.get(1).unwrap().as_str();
                    let block_content = block_cap.get(2).unwrap().as_str();
                    let placeholder = format!("{{% block {block_name} %}}...{{% endblock %}}");
                    result = result.replace(&placeholder, block_content);
                }
            }
        }

        self.render(&result, vars)
    }

    pub fn render_with_helpers(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        result = self.helpers.process(&result, vars);
        self.render(&result, vars)
    }

    fn render_variables(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let var_re = Regex::new(r"\{\{\s*([\w.]+)\s*(\|\s*[\w]+\s*)*\}\}").unwrap();
        let mut result = template.to_string();

        for cap in var_re.captures_iter(template) {
            let full_match = cap.get(0).unwrap().as_str();
            let var_name = cap.get(1).unwrap().as_str().trim();
            let filter_part = cap.get(2).map(|m| m.as_str().trim().trim_start_matches('|').trim());

            let value = vars.get(var_name).map(|s| s.as_str()).unwrap_or("");
            let final_value = if let Some(filter_name) = filter_part {
                if let Some(f) = self.filters.get(filter_name) {
                    f(value)
                } else {
                    value.to_string()
                }
            } else {
                value.to_string()
            };

            result = result.replace(full_match, &final_value);
        }

        result
    }

    fn render_nunjucks(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        result = self.render_if(&result, vars);
        result = self.render_for(&result, vars);
        result = self.render_include(&result);
        result = self.render_set(&result, vars);
        result
    }

    fn render_if(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let if_re = Regex::new(r"\{%\s*if\s+(\w+)\s*%\}(.*?)(?:\{%\s*else\s*%\}(.*?))?\{%\s*endif\s*%\}")
            .unwrap_or_else(|_| Regex::new(r"").unwrap());
        let mut result = template.to_string();

        loop {
            let new_result = if_re.replace_all(&result, |caps: &regex::Captures| {
                let cond_var = caps.get(1).unwrap().as_str();
                let truthy = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let falsy = caps.get(3).map(|m| m.as_str()).unwrap_or("");

                let cond_val = vars.get(cond_var).map(|s| s.as_str()).unwrap_or("");
                let is_truthy = !cond_val.is_empty() && cond_val != "false" && cond_val != "0" && cond_val != "no";

                if is_truthy { truthy.to_string() } else { falsy.to_string() }
            }).to_string();

            if new_result == result { break; }
            result = new_result;
        }

        result
    }

    fn render_for(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let for_re = Regex::new(r"\{%\s*for\s+(\w+)\s+in\s+(\w+)\s*%\}(.*?)\{%\s*endfor\s*%\}")
            .unwrap_or_else(|_| Regex::new(r"").unwrap());
        let mut result = template.to_string();

        loop {
            let new_result = for_re.replace_all(&result, |caps: &regex::Captures| {
                let item_var = caps.get(1).unwrap().as_str();
                let list_var = caps.get(2).unwrap().as_str();
                let body = caps.get(3).map(|m| m.as_str()).unwrap_or("");

                let list_val = vars.get(list_var).map(|s| s.as_str()).unwrap_or("");
                let items: Vec<&str> = if list_val.contains(',') {
                    list_val.split(',').map(|s| s.trim()).collect()
                } else if list_val.contains('\n') {
                    list_val.lines().collect()
                } else {
                    vec![list_val]
                };

                let mut output = String::new();
                for item in &items {
                    let mut item_vars = vars.clone();
                    item_vars.insert(item_var.to_string(), item.to_string());
                    output.push_str(&self.render(body, &item_vars));
                }
                output
            }).to_string();

            if new_result == result { break; }
            result = new_result;
        }

        result
    }

    fn render_include(&self, template: &str) -> String {
        let include_re = Regex::new(r#"\{%\s*include\s+"([^"]+)"\s*%\}"#).unwrap();
        let mut result = template.to_string();

        loop {
            let new_result = include_re.replace_all(&result, |caps: &regex::Captures| {
                let partial_name = caps.get(1).unwrap().as_str();
                self.partials.get(partial_name).cloned().unwrap_or_default()
            }).to_string();

            if new_result == result { break; }
            result = new_result;
        }

        result
    }

    fn render_set(&self, template: &str, _vars: &HashMap<String, String>) -> String {
        let set_re = Regex::new(r#"\{%\s*set\s+(\w+)\s*=\s*"([^"]+)"\s*%\}"#).unwrap();
        set_re.replace_all(template, "").to_string()
    }

    fn render_handlebars(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        result = self.render_handlebars_if(&result, vars);
        result = self.render_handlebars_each(&result, vars);
        result
    }

    fn render_handlebars_if(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let if_re = Regex::new(r"\{\{#if\s+(\w+)\s*\}\}(.*?)(?:\{\{else\}\}(.*?))?\{\{/if\}\}")
            .unwrap_or_else(|_| Regex::new(r"").unwrap());
        let mut result = template.to_string();

        loop {
            let new_result = if_re.replace_all(&result, |caps: &regex::Captures| {
                let cond_var = caps.get(1).unwrap().as_str();
                let truthy = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let falsy = caps.get(3).map(|m| m.as_str()).unwrap_or("");

                let cond_val = vars.get(cond_var).map(|s| s.as_str()).unwrap_or("");
                let is_truthy = !cond_val.is_empty() && cond_val != "false" && cond_val != "0";

                if is_truthy { truthy.to_string() } else { falsy.to_string() }
            }).to_string();

            if new_result == result { break; }
            result = new_result;
        }

        result
    }

    fn render_handlebars_each(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let each_re = Regex::new(r"\{\{#each\s+(\w+)\s*\}\}(.*?)\{\{/each\}\}")
            .unwrap_or_else(|_| Regex::new(r"").unwrap());
        let mut result = template.to_string();

        loop {
            let new_result = each_re.replace_all(&result, |caps: &regex::Captures| {
                let list_var = caps.get(1).unwrap().as_str();
                let body = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                let list_val = vars.get(list_var).map(|s| s.as_str()).unwrap_or("");
                let items: Vec<&str> = if list_val.contains(',') {
                    list_val.split(',').map(|s| s.trim()).collect()
                } else {
                    vec![list_val]
                };

                let mut output = String::new();
                for (idx, item) in items.iter().enumerate() {
                    let mut item_vars = vars.clone();
                    item_vars.insert("this".to_string(), item.to_string());
                    item_vars.insert("@index".to_string(), idx.to_string());
                    output.push_str(&self.render(body, &item_vars));
                }
                output
            }).to_string();

            if new_result == result { break; }
            result = new_result;
        }

        result
    }

    pub fn has_partial(&self, name: &str) -> bool {
        self.partials.contains_key(name)
    }

    pub fn partials(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.partials.keys().cloned().collect();
        keys.sort();
        keys
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_variable() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        assert_eq!(engine.render("Hello {{ name }}", &vars), "Hello World");
    }

    #[test]
    fn test_render_filter_upper() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "hello".to_string());
        assert_eq!(engine.render("{{ name | upper }}", &vars), "HELLO");
    }

    #[test]
    fn test_render_filter_capitalize() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "hello".to_string());
        assert_eq!(engine.render("{{ name | capitalize }}", &vars), "Hello");
    }

    #[test]
    fn test_render_nunjucks_if_true() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("show".to_string(), "true".to_string());
        assert_eq!(engine.render("{% if show %}YES{% endif %}", &vars), "YES");
    }

    #[test]
    fn test_render_nunjucks_if_false() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("show".to_string(), "".to_string());
        assert_eq!(engine.render("{% if show %}YES{% else %}NO{% endif %}", &vars), "NO");
    }

    #[test]
    fn test_render_nunjucks_for() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("items".to_string(), "a,b,c".to_string());
        let result = engine.render("{% for item in items %}{{ item }},{% endfor %}", &vars);
        assert_eq!(result, "a,b,c,");
    }

    #[test]
    fn test_render_include() {
        let mut engine = TemplateEngine::new();
        engine.add_partial("header", "<header>HEADER</header>");
        let vars = HashMap::new();
        assert_eq!(engine.render("{% include \"header\" %}", &vars), "<header>HEADER</header>");
    }

    #[test]
    fn test_handlebars_if() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("admin".to_string(), "true".to_string());
        assert_eq!(engine.render("{{#if admin}}ADMIN{{/if}}", &vars), "ADMIN");
    }

    #[test]
    fn test_handlebars_each() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("users".to_string(), "Alice,Bob,Charlie".to_string());
        let result = engine.render("{{#each users}}{{this}},{{/each}}", &vars);
        assert_eq!(result, "Alice,Bob,Charlie,");
    }

    #[test]
    fn test_custom_filter() {
        let mut engine = TemplateEngine::new();
        engine.register_filter("exclaim", |s| format!("{s}!"));
        let mut vars = HashMap::new();
        vars.insert("msg".to_string(), "hello".to_string());
        assert_eq!(engine.render("{{ msg | exclaim }}", &vars), "hello!");
    }

    #[test]
    fn test_block_and_extends() {
        let mut engine = TemplateEngine::new();
        engine.add_partial("layout", "<html>{% block content %}...{% endblock %}</html>");
        let template = "{% extends \"layout\" %}{% block content %}BODY{% endblock %}";
        let vars = HashMap::new();
        let result = engine.render_with_inheritance(template, &vars);
        assert!(result.contains("BODY"));
        assert!(result.contains("<html>"));
    }

    #[test]
    fn test_load_partials_dir() {
        let dir = std::env::temp_dir().join(format!("klyron_partials_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("header.html"), "<header>HEADER</header>").unwrap();
        std::fs::write(dir.join("footer.html"), "<footer>FOOTER</footer>").unwrap();

        let mut engine = TemplateEngine::new();
        engine.load_partials_dir(&dir).unwrap();
        assert!(engine.has_partial("header"));
        assert!(engine.has_partial("footer"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_json_filter() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("data".to_string(), "hello".to_string());
        let result = engine.render("{{ data | json }}", &vars);
        assert_eq!(result, "\"hello\"");
    }
}
