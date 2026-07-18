use std::collections::HashMap;

use regex::Regex;
use sha2::{Digest, Sha256};

pub struct TemplateHelpers {
    helpers: HashMap<String, Box<dyn Fn(&[&str]) -> String + Send + Sync>>,
}

impl Clone for TemplateHelpers {
    fn clone(&self) -> Self {
        Self {
            helpers: HashMap::new(),
        }
    }
}

impl TemplateHelpers {
    pub fn new() -> Self {
        let mut helpers = Self {
            helpers: HashMap::new(),
        };
        helpers.register_defaults();
        helpers
    }

    fn register_defaults(&mut self) {
        self.register("now", |_| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            dur.as_secs().to_string()
        });

        self.register("date", |args| {
            let format = args.first().unwrap_or(&"%Y-%m-%d");
            // simplified: just return the format hint
            format!("<date:{}>", format)
        });

        self.register("uppercase", |args| {
            args.join(" ").to_uppercase()
        });

        self.register("lowercase", |args| {
            args.join(" ").to_lowercase()
        });

        self.register("capitalize", |args| {
            let s = args.join(" ");
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        });

        self.register("slugify", |args| {
            let s = args.join(" ");
            s.to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
                .collect::<String>()
        });

        self.register("truncate", |args| {
            let s = args.first().unwrap_or(&"");
            let len = args.get(1).and_then(|l| l.parse::<usize>().ok()).unwrap_or(100);
            if s.len() > len {
                format!("{}...", &s[..len])
            } else {
                s.to_string()
            }
        });

        self.register("default", |args| {
            let value = args.first().unwrap_or(&"");
            let default = args.get(1).unwrap_or(&"default");
            if value.is_empty() { default.to_string() } else { value.to_string() }
        });

        self.register("json", |args| {
            let s = args.join(" ");
            serde_json::to_string(&s).unwrap_or_else(|_| s)
        });

        self.register("base64_encode", |args| {
            let s = args.join(" ");
            use std::io::Write;
            let mut encoder = base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
            let _ = write!(encoder, "{}", s);
            encoder.into_inner()
        });

        self.register("sha256", |args| {
            let s = args.join(" ");
            let hash = Sha256::digest(s.as_bytes());
            format!("{:x}", hash)
        });

        self.register("concat", |args| args.join(""));

        self.register("join", |args| {
            let separator = args.first().copied().unwrap_or(",");
            let rest = &args[1..];
            rest.join(separator)
        });

        self.register("first", |args| {
            args.first().unwrap_or(&"").to_string()
        });

        self.register("last", |args| {
            args.last().unwrap_or(&"").to_string()
        });

        self.register("length", |args| {
            args.first().map(|s| s.len().to_string()).unwrap_or_else(|| "0".to_string())
        });
    }

    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&[&str]) -> String + Send + Sync + 'static,
    {
        self.helpers.insert(name.to_string(), Box::new(f));
    }

    pub fn process(&self, template: &str, vars: &HashMap<String, String>) -> String {
        let re = Regex::new(r"\{\{\s*([\w_]+)\s*\(([^)]*)\)\s*\}\}").unwrap();
        let mut result = template.to_string();

        for cap in re.captures_iter(template) {
            let full_match = cap.get(0).unwrap().as_str();
            let helper_name = cap.get(1).unwrap().as_str();
            let args_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            let mut args: Vec<String> = Vec::new();
            for arg in args_str.split(',') {
                let arg = arg.trim();
                if arg.is_empty() { continue; }
                if arg.starts_with('"') && arg.ends_with('"') {
                    args.push(arg[1..arg.len()-1].to_string());
                } else if let Some(val) = vars.get(arg) {
                    args.push(val.clone());
                } else {
                    args.push(arg.to_string());
                }
            }

            if let Some(helper) = self.helpers.get(helper_name) {
                let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let output = helper(&args_refs);
                result = result.replace(full_match, &output);
            }
        }

        result
    }

    pub fn has_helper(&self, name: &str) -> bool {
        self.helpers.contains_key(name)
    }

    pub fn helper_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.helpers.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for TemplateHelpers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_now() {
        let helpers = TemplateHelpers::new();
        let result = helpers.process("{{ now() }}", &HashMap::new());
        assert!(!result.is_empty());
        assert!(!result.contains("now()"));
    }

    #[test]
    fn test_helper_uppercase() {
        let helpers = TemplateHelpers::new();
        let result = helpers.process("{{ uppercase(\"hello\") }}", &HashMap::new());
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_helper_lowercase() {
        let helpers = TemplateHelpers::new();
        let result = helpers.process("{{ lowercase(\"HELLO\") }}", &HashMap::new());
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_helper_slugify() {
        let helpers = TemplateHelpers::new();
        let result = helpers.process("{{ slugify(\"Hello World\") }}", &HashMap::new());
        assert_eq!(result, "hello-world");
    }

    #[test]
    fn test_helper_truncate() {
        let helpers = TemplateHelpers::new();
        let result = helpers.process("{{ truncate(\"hello world\", 5) }}", &HashMap::new());
        assert_eq!(result, "hello...");
    }

    #[test]
    fn test_helper_default() {
        let helpers = TemplateHelpers::new();
        let result = helpers.process("{{ default(\"\", \"fallback\") }}", &HashMap::new());
        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_helper_json() {
        let helpers = TemplateHelpers::new();
        let result = helpers.process("{{ json(\"hello\") }}", &HashMap::new());
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_helper_concat() {
        let helpers = TemplateHelpers::new();
        let result = helpers.process("{{ concat(\"a\", \"b\", \"c\") }}", &HashMap::new());
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_helper_sha256() {
        let helpers = TemplateHelpers::new();
        let result = helpers.process("{{ sha256(\"test\") }}", &HashMap::new());
        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_helper_with_vars() {
        let helpers = TemplateHelpers::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        let result = helpers.process("{{ uppercase(name) }}", &vars);
        assert_eq!(result, "ALICE");
    }

    #[test]
    fn test_helper_names() {
        let helpers = TemplateHelpers::new();
        let names = helpers.helper_names();
        assert!(names.contains(&"uppercase".to_string()));
        assert!(names.contains(&"sha256".to_string()));
    }
}
