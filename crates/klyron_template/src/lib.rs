pub mod engine;
pub mod helpers;

pub use engine::TemplateEngine;
pub use helpers::TemplateHelpers;

use std::collections::HashMap;

pub fn render(template: &str, vars: &HashMap<String, String>) -> String {
    TemplateEngine::render_static(template, vars)
}

pub fn render_with_helpers(template: &str, vars: &HashMap<String, String>) -> String {
    let engine = TemplateEngine::new();
    engine.render_with_helpers(template, vars)
}

pub fn render_inheritance(template: &str, vars: &HashMap<String, String>, partials: HashMap<String, String>) -> String {
    let mut engine = TemplateEngine::new();
    for (name, content) in partials {
        engine.add_partial(&name, &content);
    }
    engine.render_with_inheritance(template, &vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_function() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        assert_eq!(render("Hello {{ name }}", &vars), "Hello World");
    }

    #[test]
    fn test_render_with_helpers() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "world".to_string());
        let result = render_with_helpers("{{ uppercase(name) }}", &vars);
        assert_eq!(result, "WORLD");
    }

    #[test]
    fn test_render_inheritance() {
        let mut partials = HashMap::new();
        partials.insert("layout".to_string(), "<html>{% block content %}...{% endblock %}</html>".to_string());
        let template = "{% extends \"layout\" %}{% block content %}BODY{% endblock %}";
        let vars = HashMap::new();
        let result = render_inheritance(template, &vars, partials);
        assert!(result.contains("BODY"));
        assert!(result.contains("<html>"));
    }

    #[test]
    fn test_empty_vars() {
        let vars = HashMap::new();
        assert_eq!(render("Hello {{ name }}", &vars), "Hello ");
    }
}
