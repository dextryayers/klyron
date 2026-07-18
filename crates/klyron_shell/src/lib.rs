pub mod builtin;
pub mod job;
pub mod parser;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{CompletionType, Config, EditMode, Editor, Helper};
use std::borrow::Cow::{self, Borrowed, Owned};
use tracing::debug;

const GLOBAL_APIS: &[&str] = &[
    "console", "process", "fetch", "setTimeout", "setInterval",
    "clearTimeout", "clearInterval", "require", "import", "export",
    "Promise", "async", "await", "Array", "Object", "String", "Number",
    "Boolean", "Map", "Set", "Symbol", "Buffer", "globalThis", "global",
    "module", "exports", "__dirname", "__filename", "JSON", "Math",
    "Date", "RegExp", "Error", "TypeError", "ReferenceError",
    "console.log", "console.error", "console.warn", "console.info",
    "console.debug", "console.table", "console.time", "console.timeEnd",
    "process.env", "process.argv", "process.cwd", "process.exit",
    "process.nextTick", "process.stdout", "process.stderr", "process.stdin",
    "fetch", "Request", "Response", "Headers", "URL", "URLSearchParams",
    "TextEncoder", "TextDecoder", "atob", "btoa", "crypto",
    "localStorage", "sessionStorage", "clearTimeout", "clearInterval",
    "queueMicrotask", "Intl", "BigInt", "WeakMap", "WeakSet", "Proxy",
    "Reflect",
];

#[derive(Clone, Debug)]
struct ShellHelper {
    commands: Vec<String>,
}

impl ShellHelper {
    fn new() -> Self {
        Self {
            commands: vec![
                "help".into(), "exit".into(), "clear".into(),
                "history".into(), "echo".into(), "cat".into(),
                "ls".into(), "pwd".into(), "cd".into(), "type".into(),
            ],
        }
    }

    fn add_command(&mut self, cmd: &str) {
        if !self.commands.contains(&cmd.to_string()) {
            self.commands.push(cmd.to_string());
        }
    }

    fn add_global_apis(&mut self) {
        for api in GLOBAL_APIS {
            self.add_command(api);
        }
    }
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_prefix = &line[..pos];
        let last_word = line_prefix.split_whitespace().last().unwrap_or("");
        let start_pos = pos - last_word.len();

        let mut completions: Vec<Pair> = self.commands.iter()
            .filter(|c| c.starts_with(last_word) && !last_word.is_empty())
            .map(|c| Pair {
                display: c.clone(),
                replacement: c.clone(),
            })
            .collect();

        if last_word.contains('.') {
            let obj = last_word.split('.').next().unwrap_or("");
            let prefix = last_word.split('.').last().unwrap_or("");
            let completions_for_obj: &[&str] = match obj {
                "console" => &["log", "error", "warn", "info", "debug", "table", "time", "timeEnd"],
                "process" => &["env", "argv", "cwd", "exit", "nextTick", "stdout", "stderr", "stdin", "pid"],
                "Math" => &["random", "floor", "ceil", "round", "abs", "max", "min", "pow", "sqrt", "PI", "E"],
                "JSON" => &["parse", "stringify"],
                _ => &[],
            };
            for c in completions_for_obj {
                if c.starts_with(prefix) {
                    completions.push(Pair {
                        display: format!("{}.{}", obj, c),
                        replacement: format!("{}.{}", obj, c),
                    });
                }
            }
            return Ok((start_pos - obj.len() - 1, completions));
        }

        completions.sort_by(|a, b| a.replacement.cmp(&b.replacement));
        completions.dedup_by(|a, b| a.replacement == b.replacement);

        if completions.is_empty() && !last_word.is_empty() {
            for candidate in self.commands.iter() {
                if candidate.to_lowercase().contains(&last_word.to_lowercase()) {
                    completions.push(Pair {
                        display: candidate.clone(),
                        replacement: candidate.clone(),
                    });
                }
            }
        }

        Ok((start_pos, completions))
    }
}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for ShellHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            return Owned(format!("\x1b[90m{}\x1b[0m", line));
        }
        Owned(line.to_string())
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _app_char: bool) -> bool {
        true
    }

    fn highlight_hint<'b>(&self, hint: &'b str) -> Cow<'b, str> {
        Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }
}

impl Validator for ShellHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();
        if input.is_empty() {
            return Ok(ValidationResult::Valid(None));
        }
        if input.trim().ends_with('\\') {
            return Ok(ValidationResult::Incomplete);
        }

        let mut paren_depth = 0i64;
        let mut brace_depth = 0i64;
        let mut bracket_depth = 0i64;
        let mut in_single = false;
        let mut in_double = false;
        let mut in_backtick = false;

        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let ch = chars[i];
            if ch == '\\' && (in_single || in_double || in_backtick) {
                i += 2; continue;
            }
            if ch == '\'' && !in_double && !in_backtick { in_single = !in_single; i += 1; continue; }
            if ch == '"' && !in_single && !in_backtick { in_double = !in_double; i += 1; continue; }
            if ch == '`' && !in_single && !in_double { in_backtick = !in_backtick; i += 1; continue; }
            if !in_single && !in_double && !in_backtick {
                match ch {
                    '(' => paren_depth += 1,
                    ')' => paren_depth -= 1,
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    '[' => bracket_depth += 1,
                    ']' => bracket_depth -= 1,
                    _ => {}
                }
            }
            i += 1;
        }

        if in_single || in_double || in_backtick {
            return Ok(ValidationResult::Incomplete);
        }
        if paren_depth > 0 || brace_depth > 0 || bracket_depth > 0 {
            return Ok(ValidationResult::Incomplete);
        }

        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for ShellHelper {}

pub struct ShellRepl {
    editor: Editor<ShellHelper, FileHistory>,
    commands: HashMap<String, Arc<dyn Fn(&[String]) -> Result<String> + Send + Sync>>,
    builtins: builtin::BuiltinRegistry,
    job_manager: job::JobManager,
}

impl Default for ShellRepl {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellRepl {
    pub fn new() -> Self {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .max_history_size(1000)
            .expect("Failed to set max history size")
            .auto_add_history(true)
            .build();

        let mut helper = ShellHelper::new();
        helper.add_global_apis();

        let mut editor: Editor<ShellHelper, FileHistory> = Editor::with_config(config)
            .expect("Failed to create rustyline editor");
        editor.set_helper(Some(helper));

        if let Err(e) = editor.load_history(".klyron_history") {
            debug!("No previous history found: {}", e);
        }

        let mut repl = Self {
            editor,
            commands: HashMap::new(),
            builtins: builtin::BuiltinRegistry::new(),
            job_manager: job::JobManager::new(),
        };

        repl.register_builtins();
        repl
    }

    fn register_builtins(&mut self) {
        self.register_command("help", {
            let builtins = self.builtins.names();
            move |args| {
                if args.is_empty() {
                    Ok(format!("Available commands:\n  {}\n\nType 'help <cmd>' for details.",
                        builtins.join("\n  ")))
                } else {
                    Ok(format!("Help for '{}': built-in command", args[0]))
                }
            }
        });

        self.register_command("exit", |_| { std::process::exit(0); });
        self.register_command("clear", |_| { print!("\x1b[2J\x1b[H"); Ok(String::new()) });
        self.register_command("echo", |args| Ok(args.join(" ")));
        self.register_command("history", |_| Ok("Use up/down arrows to navigate history".to_string()));
        self.register_command("pwd", |_| {
            Ok(std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".to_string()))
        });
        self.register_command("cd", |args| {
            let dir = if args.is_empty() {
                std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from("/"))
            } else {
                std::path::PathBuf::from(&args[0])
            };
            std::env::set_current_dir(&dir).map_err(|e| anyhow::anyhow!("cd: {}", e))?;
            Ok(String::new())
        });
        self.register_command("ls", |args| {
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
        self.register_command("jobs", |_| {
            Ok("Use 'ps' or 'jobs' is not available in this mode".to_string())
        });
    }

    pub fn register_command<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&[String]) -> Result<String> + Send + Sync + 'static,
    {
        self.commands.insert(name.to_string(), Arc::new(handler));
        if let Some(helper) = self.editor.helper_mut() {
            helper.add_command(name);
        }
    }

    pub fn history(&self) -> Vec<String> {
        self.editor.history().iter().map(|e| e.to_string()).collect()
    }

    pub fn eval(&mut self, input: &str) -> Result<String> {
        let input = input.trim();
        if input.is_empty() || input.starts_with('#') || input.starts_with("//") {
            return Ok(String::new());
        }

        let jobs = parser::parse_line(input)?;
        let mut results = Vec::new();

        for parsed_job in &jobs {
            let cmd = &parsed_job.pipeline.commands[0];
            if let Some(handler) = self.commands.get(cmd.program.as_str()) {
                let result = handler(&cmd.args)?;
                results.push(result);
            } else if self.builtins.contains(&cmd.program) {
                let result = self.builtins.exec(&cmd.program, &cmd.args)?;
                results.push(result);
            } else {
                let id = self.job_manager.create(input, parsed_job.pipeline.background);
                match job::run_job(parsed_job) {
                    Ok(child) => {
                        self.job_manager.set_pid(id, child.id());
                        if parsed_job.pipeline.background {
                            results.push(format!("[{}] {}", id, child.id()));
                        } else {
                            let output = child.wait_with_output()?;
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            if !stdout.is_empty() { results.push(stdout); }
                            if !stderr.is_empty() { results.push(stderr); }
                            self.job_manager.set_state(id, job::JobState::Done);
                        }
                    }
                    Err(e) => {
                        results.push(format!("error: {}", e));
                    }
                }
            }
        }

        Ok(results.join("\n"))
    }

    pub fn run(&mut self) -> Result<()> {
        println!("Klyron shell. Type 'help' for available commands.");

        loop {
            let prompt = format!("\x1b[32m❯\x1b[0m ");

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    match self.eval(&line) {
                        Ok(output) => {
                            if !output.is_empty() {
                                println!("{}", output);
                            }
                        }
                        Err(e) => {
                            eprintln!("\x1b[31merror\x1b[0m: {}", e);
                        }
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("exit");
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => break,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }

        if let Err(e) = self.editor.save_history(".klyron_history") {
            debug!("Failed to save history: {}", e);
        }

        Ok(())
    }

    pub fn run_eval_line(&mut self, code: &str) -> Result<String> {
        self.eval(code)
    }
}

pub fn colorize_output(output: &str) -> String {
    let mut result = String::with_capacity(output.len() * 2);
    let chars: Vec<char> = output.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' || ch == '\'' || ch == '`' {
            let quote = ch;
            let mut end = i + 1;
            while end < chars.len() && chars[end] != quote {
                if chars[end] == '\\' { end += 1; }
                end += 1;
            }
            if end < chars.len() { end += 1; }
            result.push_str(&format!("\x1b[32m{}\x1b[0m", &chars[i..end].iter().collect::<String>()));
            i = end;
            continue;
        }
        if ch.is_ascii_digit() || (ch == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let mut end = i;
            if chars[end] == '-' { end += 1; }
            while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.' || chars[end] == 'e' || chars[end] == 'E') { end += 1; }
            if end > i {
                result.push_str(&format!("\x1b[94m{}\x1b[0m", &chars[i..end].iter().collect::<String>()));
                i = end; continue;
            }
        }
        match ch {
            '{' | '}' | '[' | ']' | '(' | ')' => {
                result.push_str(&format!("\x1b[93m{}\x1b[0m", ch));
            }
            _ => { result.push(ch); }
        }
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_new() {
        let shell = ShellRepl::new();
        assert!(shell.commands.contains_key("help"));
        assert!(shell.commands.contains_key("exit"));
        assert!(shell.commands.contains_key("echo"));
    }

    #[test]
    fn test_eval_empty() {
        let mut shell = ShellRepl::new();
        assert_eq!(shell.eval("").unwrap(), "");
        assert_eq!(shell.eval("  ").unwrap(), "");
    }

    #[test]
    fn test_eval_comment() {
        let mut shell = ShellRepl::new();
        assert_eq!(shell.eval("# comment").unwrap(), "");
        assert_eq!(shell.eval("// comment").unwrap(), "");
    }

    #[test]
    fn test_eval_echo() {
        let mut shell = ShellRepl::new();
        let result = shell.eval("echo hello world").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_eval_pwd() {
        let mut shell = ShellRepl::new();
        let result = shell.eval("pwd").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_eval_unknown() {
        let mut shell = ShellRepl::new();
        let result = shell.eval("nonexistent_cmd").unwrap();
        assert!(result.contains("error:"));
    }

    #[test]
    fn test_colorize_output() {
        let colored = colorize_output("hello \"world\" 42");
        assert!(colored.contains("\x1b[32m"));
        assert!(colored.contains("\x1b[0m"));
    }

    #[test]
    fn test_parser_integration() {
        let jobs = parser::parse_line("echo hello | wc -c").unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].pipeline.commands.len(), 2);
    }

    #[test]
    fn test_builtin_integration() {
        let mut shell = ShellRepl::new();
        let result = shell.eval("echo test").unwrap();
        assert_eq!(result, "test");
    }
}
