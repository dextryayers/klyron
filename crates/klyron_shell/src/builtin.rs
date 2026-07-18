use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

type BuiltinHandler = Arc<dyn Fn(&[String]) -> Result<String> + Send + Sync>;

pub struct BuiltinRegistry {
    handlers: HashMap<String, BuiltinHandler>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            handlers: HashMap::new(),
        };
        reg.register_defaults();
        reg
    }

    fn register_defaults(&mut self) {
        self.register("help", |args| {
            let text = if args.is_empty() {
                r#"Built-in commands:
  help [cmd]     - Show help
  exit           - Exit the shell
  clear          - Clear screen
  echo           - Print text
  pwd            - Print working directory
  cd [dir]       - Change directory
  ls [dir]       - List directory contents
  history        - Show command history
  type [cmd]     - Show command type"#.to_string()
            } else {
                format!("Help for '{}': built-in command", args[0])
            };
            Ok(text)
        });

        self.register("exit", |_| {
            std::process::exit(0);
        });

        self.register("clear", |_| {
            print!("\x1b[2J\x1b[H");
            Ok(String::new())
        });

        self.register("echo", |args| Ok(args.join(" ")));

        self.register("pwd", |_| {
            Ok(std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string()))
        });

        self.register("cd", |args| {
            let dir = if args.is_empty() {
                std::env::var("HOME")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| std::path::PathBuf::from("/"))
            } else {
                std::path::PathBuf::from(&args[0])
            };
            std::env::set_current_dir(&dir)
                .map_err(|e| anyhow::anyhow!("cd: {}", e))?;
            Ok(String::new())
        });

        self.register("ls", |args| {
            let dir = if args.is_empty() {
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
            } else {
                std::path::PathBuf::from(&args[0])
            };
            let mut output = String::new();
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        output.push_str(&format!("\x1b[34m{}/\x1b[0m  ", name));
                    } else {
                        output.push_str(&format!("{}  ", name));
                    }
                }
            }
            Ok(output)
        });

        self.register("history", |_| {
            Ok("History is managed by the shell".to_string())
        });

        self.register("type", |args| {
            if args.is_empty() {
                return Ok("Usage: type <command>".to_string());
            }
            Ok(format!("{} is a built-in command", args[0]))
        });
    }

    pub fn register<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&[String]) -> Result<String> + Send + Sync + 'static,
    {
        self.handlers.insert(name.to_string(), Arc::new(handler));
    }

    pub fn get(&self, name: &str) -> Option<&BuiltinHandler> {
        self.handlers.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.handlers.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn exec(&self, name: &str, args: &[String]) -> Result<String> {
        match self.handlers.get(name) {
            Some(handler) => handler(args),
            None => anyhow::bail!("unknown built-in: {}", name),
        }
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_registry_contains() {
        let reg = BuiltinRegistry::new();
        assert!(reg.contains("echo"));
        assert!(reg.contains("cd"));
        assert!(reg.contains("pwd"));
        assert!(reg.contains("exit"));
    }

    #[test]
    fn test_builtin_echo() {
        let reg = BuiltinRegistry::new();
        let result = reg.exec("echo", &["hello".into(), "world".into()]).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_builtin_unknown() {
        let reg = BuiltinRegistry::new();
        assert!(reg.exec("nonexistent", &[]).is_err());
    }

    #[test]
    fn test_builtin_names() {
        let reg = BuiltinRegistry::new();
        let names = reg.names();
        assert!(names.contains(&"echo".to_string()));
        assert!(names.contains(&"cd".to_string()));
    }

    #[test]
    fn test_custom_builtin() {
        let mut reg = BuiltinRegistry::new();
        reg.register("greet", |args| Ok(format!("Hello, {}!", args.join(" "))));
        assert!(reg.contains("greet"));
        let result = reg.exec("greet", &["World".into()]).unwrap();
        assert_eq!(result, "Hello, World!");
    }
}
