use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// PSR-4 autoloader configuration
#[derive(Debug, Clone)]
pub struct Psr4Autoloader {
    pub prefixes: BTreeMap<String, String>,
    pub base_path: PathBuf,
}

impl Psr4Autoloader {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            prefixes: BTreeMap::new(),
            base_path: base_path.into(),
        }
    }

    /// Add a PSR-4 namespace prefix mapping
    pub fn add_prefix(&mut self, namespace: &str, directory: &str) {
        self.prefixes.insert(namespace.to_string(), directory.to_string());
    }

    /// Resolve a fully qualified class name to a file path
    pub fn resolve(&self, class_name: &str) -> Option<PathBuf> {
        let class_name = class_name.trim_start_matches('\\');

        for (prefix, directory) in &self.prefixes {
            let prefix = prefix.trim_end_matches('\\');
            if class_name.starts_with(prefix) {
                let relative_class = class_name.trim_start_matches(prefix).trim_start_matches('\\');
                if relative_class.is_empty() {
                    continue;
                }
                let file_name = relative_class.replace('\\', "/");
                let path = self.base_path.join(directory).join(format!("{}.php", file_name));
                return Some(path);
            }
        }
        None
    }

    /// Generate the autoloader PHP code
    pub fn generate_php(&self) -> String {
        let mut code = String::from("<?php\n\n");
        code.push_str("/**\n");
        code.push_str(" * Klyron PSR-4 Autoloader\n");
        code.push_str(" * Generated automatically. Do not edit.\n");
        code.push_str(" */\n\n");
        code.push_str("spl_autoload_register(function ($class) {\n");

        for (prefix, directory) in &self.prefixes {
            let prefix_escaped = prefix.replace('\\', "\\\\");
            let directory_escaped = directory.replace('/', std::path::MAIN_SEPARATOR_STR);

            code.push_str(&format!(
                "    // {} -> {}\n",
                prefix, directory
            ));
            code.push_str(&format!(
                "    $prefix = '{}';\n",
                prefix_escaped
            ));
            code.push_str(&format!(
                "    $base_dir = __DIR__ . '/{}';\n",
                directory_escaped
            ));
            code.push_str("    $len = strlen($prefix);\n");
            code.push_str("    if (strncmp($prefix, $class, $len) !== 0) {\n");
            code.push_str("        return;\n");
            code.push_str("    }\n");
            code.push_str("    $relative_class = substr($class, $len);\n");
            code.push_str("    $file = $base_dir . str_replace('\\\\', '/', $relative_class) . '.php';\n");
            code.push_str("    if (file_exists($file)) {\n");
            code.push_str("        require $file;\n");
            code.push_str("    }\n");
        }

        code.push_str("});\n");
        code
    }

    /// Scan a directory to find all PHP classes and generate namespace mappings
    pub fn scan_directory(dir: impl AsRef<Path>, base_namespace: &str) -> Result<Self, String> {
        let mut autoloader = Self::new(dir.as_ref().to_path_buf());
        let base = dir.as_ref().to_path_buf();

        autoloader.scan_recursive(&base, &base, base_namespace)?;

        Ok(autoloader)
    }

    fn scan_recursive(
        &mut self,
        base: &Path,
        current: &Path,
        namespace: &str,
    ) -> Result<(), String> {
        let entries = std::fs::read_dir(current)
            .map_err(|e| format!("Cannot read directory {}: {}", current.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| "Invalid directory name".to_string())?;
                let sub_namespace = format!("{}\\{}", namespace, dir_name);
                self.scan_recursive(base, &path, &sub_namespace)?;
            }
        }

        Ok(())
    }
}

/// Generate a composer.json autoload section from a PSR-4 autoloader
pub fn generate_autoload_section(autoloader: &Psr4Autoloader) -> serde_json::Value {
    let mut psr4 = serde_json::Map::new();
    for (prefix, dir) in &autoloader.prefixes {
        psr4.insert(
            prefix.clone(),
            serde_json::Value::String(dir.clone()),
        );
    }
    serde_json::json!({
        "psr-4": psr4
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psr4_resolve() {
        let mut autoloader = Psr4Autoloader::new("/project");
        autoloader.add_prefix("App\\", "src/");

        let path = autoloader.resolve("App\\Models\\User");
        assert!(path.is_some());
        let path_str = path.unwrap().to_string_lossy().to_string();
        assert!(path_str.contains("src/"));
        assert!(path_str.ends_with("Models/User.php"));
    }

    #[test]
    fn test_psr4_resolve_no_match() {
        let mut autoloader = Psr4Autoloader::new("/project");
        autoloader.add_prefix("App\\", "src/");

        let path = autoloader.resolve("Other\\Class");
        assert!(path.is_none());
    }

    #[test]
    fn test_autoloader_generates_php() {
        let mut autoloader = Psr4Autoloader::new("/project");
        autoloader.add_prefix("App\\", "src/");
        autoloader.add_prefix("Tests\\", "tests/");

        let php = autoloader.generate_php();
        assert!(php.contains("App\\\\"));
        assert!(php.contains("Tests\\\\"));
        assert!(php.contains("spl_autoload_register"));
        assert!(php.contains("require $file"));
    }

    #[test]
    fn test_generate_autoload_section() {
        let mut autoloader = Psr4Autoloader::new("/project");
        autoloader.add_prefix("App\\", "src/");

        let section = generate_autoload_section(&autoloader);
        assert_eq!(section["psr-4"]["App\\"], "src/");
    }
}
