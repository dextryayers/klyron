use anyhow::Result;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{CompletionType, Config, EditMode, Editor, Helper};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

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
    "localStorage", "sessionStorage", "setTimeout", "setInterval",
    "clearTimeout", "clearInterval", "queueMicrotask",
    "Intl", "BigInt", "Symbol", "WeakMap", "WeakSet", "Proxy", "Reflect",
];

#[derive(Clone, Debug)]
struct ShellHelper {
    commands: Vec<String>,
    is_multiline: bool,
}

impl ShellHelper {
    fn new() -> Self {
        Self {
            commands: vec![
                "help".into(),
                "exit".into(),
                "clear".into(),
                "history".into(),
                "echo".into(),
                "cat".into(),
                "ls".into(),
                "pwd".into(),
                "cd".into(),
            ],
            is_multiline: false,
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

        let mut completions = Vec::new();

        let candidates: Vec<&str> = self.commands.iter().map(|s| s.as_str()).collect();
        for candidate in candidates {
            if candidate.starts_with(last_word) && !last_word.is_empty() {
                completions.push(Pair {
                    display: candidate.to_string(),
                    replacement: candidate.to_string(),
                });
            }
        }

        if last_word.contains('.') {
            let obj = last_word.split('.').next().unwrap_or("");
            let prefix = last_word.split('.').last().unwrap_or("");
            let completions_for_obj: &[&str] = match obj {
                "console" => &["log", "error", "warn", "info", "debug", "table", "time", "timeEnd", "assert", "group", "groupEnd", "count", "trace", "dir"],
                "process" => &["env", "argv", "cwd", "exit", "nextTick", "stdout", "stderr", "stdin", "pid", "ppid", "platform", "arch", "version", "versions", "memoryUsage", "uptime", "hrtime"],
                "Math" => &["random", "floor", "ceil", "round", "abs", "max", "min", "pow", "sqrt", "sin", "cos", "tan", "PI", "E", "LN2", "LN10", "LOG2E", "LOG10E", "SQRT1_2", "SQRT2"],
                "JSON" => &["parse", "stringify"],
                "Array" => &["from", "isArray", "of", "prototype.push", "prototype.pop", "prototype.map", "prototype.filter", "prototype.reduce", "prototype.find", "prototype.forEach", "prototype.some", "prototype.every", "prototype.includes"],
                "Object" => &["keys", "values", "entries", "assign", "create", "defineProperty", "defineProperties", "freeze", "seal", "is", "isFrozen", "isSealed", "prototype.hasOwnProperty"],
                "String" => &["fromCharCode", "fromCodePoint", "prototype.toLowerCase", "prototype.toUpperCase", "prototype.trim", "prototype.split", "prototype.replace", "prototype.includes", "prototype.startsWith", "prototype.endsWith", "prototype.slice", "prototype.substring", "prototype.indexOf", "prototype.padStart", "prototype.padEnd"],
                "Number" => &["isNaN", "isFinite", "isInteger", "isSafeInteger", "parseInt", "parseFloat", "MAX_VALUE", "MIN_VALUE", "NaN", "NEGATIVE_INFINITY", "POSITIVE_INFINITY", "EPSILON", "MAX_SAFE_INTEGER", "MIN_SAFE_INTEGER"],
                "URL" => &["createObjectURL", "revokeObjectURL", "prototype.href", "prototype.pathname", "prototype.search", "prototype.hash", "prototype.host", "prototype.hostname", "prototype.port", "prototype.protocol", "prototype.searchParams"],
                _ => &[],
            };
            for c in completions_for_obj {
                if c.starts_with(prefix) {
                    let full = format!("{}.{}", obj, c);
                    completions.push(Pair {
                        display: full.clone(),
                        replacement: full,
                    });
                }
            }
            return Ok((start_pos - obj.len() - 1, completions));
        }

        completions.sort_by(|a, b| a.replacement.cmp(&b.replacement));
        completions.dedup_by(|a, b| a.replacement == b.replacement);

        if completions.is_empty() && !last_word.is_empty() {
            for candidate in self.commands.iter() {
                let lower_candidate = candidate.to_lowercase();
                let lower_word = last_word.to_lowercase();
                if lower_candidate.contains(&lower_word) {
                    completions.push(Pair {
                        display: candidate.to_string(),
                        replacement: candidate.to_string(),
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

        if trimmed == "exit" || trimmed == ".exit" || trimmed == ".quit" {
            return Owned(format!("\x1b[31m{}\x1b[0m", line));
        }

        let mut result = String::with_capacity(line.len() * 2);
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '/' && i + 1 < chars.len() {
                if chars[i + 1] == '/' {
                    let rest: String = chars[i..].iter().collect();
                    result.push_str(&format!("\x1b[90m{}\x1b[0m", rest));
                    break;
                }
                if chars[i + 1] == '*' {
                    let mut end = i + 2;
                    while end + 1 < chars.len() && !(chars[end] == '*' && chars[end + 1] == '/') {
                        end += 1;
                    }
                    if end + 1 < chars.len() { end += 2; }
                    let comment: String = chars[i..end].iter().collect();
                    result.push_str(&format!("\x1b[90m{}\x1b[0m", comment));
                    i = end;
                    continue;
                }
            }

            if ch == '"' || ch == '\'' || ch == '`' {
                let quote = ch;
                let mut end = i + 1;
                while end < chars.len() && chars[end] != quote {
                    if chars[end] == '\\' { end += 1; }
                    end += 1;
                }
                if end < chars.len() { end += 1; }
                let string: String = chars[i..end].iter().collect();
                result.push_str(&format!("\x1b[32m{}\x1b[0m", string));
                i = end;
                continue;
            }

            if ch.is_ascii_digit() || (ch == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
                let mut end = i;
                while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.' || chars[end] == 'x' || chars[end] == 'X' || (chars[end] >= 'a' && chars[end] <= 'f') || (chars[end] >= 'A' && chars[end] <= 'F')) {
                    end += 1;
                }
                let num: String = chars[i..end].iter().collect();
                result.push_str(&format!("\x1b[94m{}\x1b[0m", num));
                i = end;
                continue;
            }

            if ch == '/' && i + 1 < chars.len() && chars[i + 1] != '/' && chars[i + 1] != '*' {
                let mut end = i + 1;
                while end < chars.len() && chars[end] != '/' {
                    if chars[end] == '\\' { end += 1; }
                    end += 1;
                }
                if end < chars.len() { end += 1; }
                let regex: String = chars[i..end].iter().collect();
                result.push_str(&format!("\x1b[91m{}\x1b[0m", regex));
                i = end;
                continue;
            }

            if ch == '{' || ch == '}' || ch == '(' || ch == ')' || ch == '[' || ch == ']' {
                result.push_str(&format!("\x1b[93m{}\x1b[0m", ch));
                i += 1;
                continue;
            }

            if ch == 'f' && line[i..].starts_with("function") {
                let end = i + 8;
                result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end]));
                i = end;
                continue;
            }

            if ch == 'c' && line[i..].starts_with("const ") {
                let end = i + 6;
                result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end]));
                i = end;
                continue;
            }
            if ch == 'l' && line[i..].starts_with("let ") {
                let end = i + 4;
                result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end]));
                i = end;
                continue;
            }
            if ch == 'v' && line[i..].starts_with("var ") {
                let end = i + 4;
                result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end]));
                i = end;
                continue;
            }
            if ch == 'r' && line[i..].starts_with("return ") {
                let end = i + 7;
                result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end]));
                i = end;
                continue;
            }
            if ch == 'i' {
                if line[i..].starts_with("if ") || line[i..].starts_with("if(") {
                    let end = if line[i..].starts_with("if(") { i + 2 } else { i + 2 };
                    result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..=i+1]));
                    i = end;
                    continue;
                }
                if line[i..].starts_with("import ") || line[i..].starts_with("import{") {
                    let end = i + 6;
                    result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end]));
                    i = end;
                    continue;
                }
            }
            if ch == 'e' {
                if line[i..].starts_with("else ") || line[i..].starts_with("else{") {
                    let end = i + 4;
                    result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end]));
                    i = end;
                    continue;
                }
                if line[i..].starts_with("export ") || line[i..].starts_with("export{") {
                    let end = i + 6;
                    result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end]));
                    i = end;
                    continue;
                }
            }
            if ch == 'n' && line[i..].starts_with("new ") {
                let end = i + 4;
                result.push_str(&format!("\x1b[95m{}\x1b[0m", &line[i..end]));
                i = end;
                continue;
            }
            if ch == 't' {
                if line[i..].starts_with("true") { let end = i + 4; result.push_str(&format!("\x1b[95m{}\x1b[0m", &line[i..end])); i = end; continue; }
                if line[i..].starts_with("throw ") { let end = i + 6; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
                if line[i..].starts_with("typeof ") { let end = i + 7; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
                if line[i..].starts_with("try ") || line[i..].starts_with("try{") { let end = i + 3; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            }
            if ch == 'f' && line[i..].starts_with("false") { let end = i + 5; result.push_str(&format!("\x1b[95m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'n' && line[i..].starts_with("null") { let end = i + 4; result.push_str(&format!("\x1b[95m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'u' && line[i..].starts_with("undefined") { let end = i + 9; result.push_str(&format!("\x1b[95m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'a' && line[i..].starts_with("async ") { let end = i + 6; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'a' && line[i..].starts_with("await ") { let end = i + 6; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'c' && line[i..].starts_with("class ") { let end = i + 6; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'd' && line[i..].starts_with("delete ") { let end = i + 7; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 's' && line[i..].starts_with("switch ") { let end = i + 7; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'w' && line[i..].starts_with("while ") { let end = i + 6; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'f' && line[i..].starts_with("for ") { let end = i + 4; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'i' && line[i..].starts_with("in ") { let end = i + 3; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'o' && line[i..].starts_with("of ") { let end = i + 3; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'c' && line[i..].starts_with("catch ") { let end = i + 6; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'f' && line[i..].starts_with("finally ") { let end = i + 8; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'd' && line[i..].starts_with("do ") { let end = i + 3; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'y' && line[i..].starts_with("yield ") { let end = i + 6; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'd' && line[i..].starts_with("debugger") { let end = i + 8; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'c' && line[i..].starts_with("case ") { let end = i + 5; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end])); i = end; continue; }
            if ch == 'd' && line[i..].starts_with("default") { let end = i + 7; let end2 = if end < chars.len() && chars[end] == ':' { end + 1 } else { end }; result.push_str(&format!("\x1b[96m{}\x1b[0m", &line[i..end2])); i = end2; continue; }

            if ch == ' ' || ch == '\t' {
                result.push(ch);
                i += 1;
                continue;
            }

            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                let mut end = i;
                while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_' || chars[end] == '$') {
                    end += 1;
                }
                let word: String = chars[i..end].iter().collect();
                if GLOBAL_APIS.contains(&word.as_str()) || (word.contains('.') && word.split('.').all(|part| GLOBAL_APIS.iter().any(|g| g == &part) || part.chars().all(|c| c.is_alphanumeric() || c == '_'))) {
                    result.push_str(&format!("\x1b[33m{}\x1b[0m", word));
                } else {
                    result.push_str(&word);
                }
                i = end;
                continue;
            }

            result.push(ch);
            i += 1;
        }

        Owned(result)
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
        let mut in_line_comment = false;
        let mut in_block_comment = false;

        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if in_line_comment {
                i += 1;
                continue;
            }
            if in_block_comment {
                if ch == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    in_block_comment = false;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }

            if ch == '/' && i + 1 < chars.len() {
                if chars[i + 1] == '/' {
                    in_line_comment = true;
                    i += 2;
                    continue;
                }
                if chars[i + 1] == '*' {
                    in_block_comment = true;
                    i += 2;
                    continue;
                }
            }

            if ch == '\\' && (in_single || in_double || in_backtick) {
                i += 2;
                continue;
            }

            if ch == '\'' && !in_double && !in_backtick {
                in_single = !in_single;
                i += 1;
                continue;
            }
            if ch == '"' && !in_single && !in_backtick {
                in_double = !in_double;
                i += 1;
                continue;
            }
            if ch == '`' && !in_single && !in_double {
                in_backtick = !in_backtick;
                i += 1;
                continue;
            }

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

        if in_single || in_double || in_backtick || in_block_comment {
            return Ok(ValidationResult::Incomplete);
        }

        if paren_depth > 0 || brace_depth > 0 || bracket_depth > 0 {
            return Ok(ValidationResult::Incomplete);
        }

        let trimmed = input.trim();
        if trimmed.ends_with(',') || trimmed.ends_with("&&") || trimmed.ends_with("||") || trimmed.ends_with('+') || trimmed.ends_with('-') || trimmed.ends_with('*') || trimmed.ends_with('/') || trimmed.ends_with('%') || trimmed.ends_with('=') || trimmed.ends_with('>') || trimmed.ends_with('<') || trimmed.ends_with('!') || trimmed.ends_with('?') || trimmed.ends_with(':') {
            return Ok(ValidationResult::Incomplete);
        }

        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for ShellHelper {}

pub struct ShellRepl {
    editor: Editor<ShellHelper, FileHistory>,
    commands: HashMap<String, Arc<dyn Fn(&[String]) -> Result<String> + Send + Sync>>,
    multiline_buffer: String,
    eval_mode: bool,
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
            multiline_buffer: String::new(),
            eval_mode: false,
        };

        repl.register_builtins();
        repl
    }

    pub fn with_eval_mode(mut self) -> Self {
        self.eval_mode = true;
        self
    }

    pub fn eval_mode(&self) -> bool {
        self.eval_mode
    }

    fn register_builtins(&mut self) {
        self.register_command("help", |args| {
            let help_text = if args.is_empty() {
                r#"Available commands:
  help [cmd]     - Show this help or help for a specific command
  exit           - Exit the shell
  clear          - Clear the screen
  history        - Show command history
  echo           - Echo text
  eval           - Evaluate a JavaScript/TypeScript expression
  ls             - List directory contents
  pwd            - Print working directory
  cd             - Change directory

Tip: Tab to auto-complete commands and global APIs.
     Up/Down arrows to navigate history.
     Multi-line: unclosed {, (, [ will continue input."#
                    .to_string()
            } else {
                format!("Help for '{}': built-in command", args[0])
            };
            Ok(help_text)
        });

        self.register_command("exit", |_| {
            std::process::exit(0);
        });

        self.register_command("clear", |_| {
            print!("\x1b[2J\x1b[H");
            Ok(String::new())
        });

        self.register_command("echo", |args| Ok(args.join(" ")));

        self.register_command("history", |_| {
            Ok("Use up/down arrows to navigate history. History saved to .klyron_history".to_string())
        });

        self.register_command("eval", |args| {
            if args.is_empty() {
                Ok("Usage: eval <expression>".to_string())
            } else {
                Ok(format!("=> {}", args.join(" ")))
            }
        });

        self.register_command("pwd", |_| {
            let cwd = std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            Ok(cwd)
        });

        self.register_command("cd", |args| {
            let dir = if args.is_empty() {
                std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_else(|_| std::path::PathBuf::from("/"))
            } else {
                std::path::PathBuf::from(&args[0])
            };
            match std::env::set_current_dir(&dir) {
                Ok(_) => Ok(String::new()),
                Err(e) => Err(anyhow::anyhow!("cd: {}", e)),
            }
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
                    } else if entry.path().extension().map(|e| e == "js" || e == "ts" || e == "jsx" || e == "tsx").unwrap_or(false) {
                        output.push_str(&format!("\x1b[32m{}\x1b[0m  ", name));
                    } else {
                        output.push_str(&format!("{}  ", name));
                    }
                }
            }
            Ok(output)
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
        self.editor
            .history()
            .iter()
            .map(|e| e.to_string())
            .collect()
    }

    pub fn eval(&mut self, input: &str) -> Result<String> {
        let input = input.trim();

        if input.is_empty() {
            return Ok(String::new());
        }

        if input.starts_with('#') || input.starts_with("//") {
            return Ok(String::new());
        }

        let parts: Vec<&str> = if input.contains('|') {
            input.split('|').map(|s| s.trim()).collect()
        } else {
            vec![input]
        };

        if parts.len() > 1 {
            return self.eval_pipeline(&parts);
        }

        let tokens: Vec<String> = shell_words_split(input);
        if tokens.is_empty() {
            return Ok(String::new());
        }

        let command = &tokens[0];
        let args = &tokens[1..];

        if let Some(handler) = self.commands.get(command.as_str()) {
            handler(args)
        } else {
            Ok(format!(
                "Unknown command: {}. Type 'help' for available commands.",
                command
            ))
        }
    }

    fn eval_pipeline(&mut self, parts: &[&str]) -> Result<String> {
        let mut previous_output = String::new();
        let mut results = Vec::new();

        for (i, part) in parts.iter().enumerate() {
            let mut cmd_input = part.to_string();
            if !previous_output.is_empty() {
                cmd_input = format!("{} {}", cmd_input, previous_output.trim());
            }

            let tokens: Vec<String> = shell_words_split(&cmd_input);
            if tokens.is_empty() {
                continue;
            }

            let command = &tokens[0];
            let args = &tokens[1..];

            let result = if let Some(handler) = self.commands.get(command.as_str()) {
                handler(args)
            } else {
                Ok(format!("<pipe {}> {}", i + 1, cmd_input))
            };

            match result {
                Ok(output) => {
                    previous_output = output;
                    results.push(previous_output.clone());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(results.join("\n"))
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Starting Klyron shell. Type 'help' for available commands.");

        loop {
            let prompt = format!("\x1b[32m❯\x1b[0m ");

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    if self.eval_mode {
                        if let Err(e) = self.editor.add_history_entry(line.as_str()) {
                            debug!("Failed to add history: {}", e);
                        }
                    }
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
                Err(rustyline::error::ReadlineError::Eof) => {
                    break;
                }
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

fn shell_words_split(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if !in_single => {
                escaped = true;
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
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
            let s: String = chars[i..end].iter().collect();
            result.push_str(&format!("\x1b[32m{}\x1b[0m", s));
            i = end;
            continue;
        }

        if ch.is_ascii_digit() || (ch == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let mut end = i;
            if chars[end] == '-' { end += 1; }
            while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.' || chars[end] == 'e' || chars[end] == 'E' || chars[end] == '+' || chars[end] == '-') {
                if (chars[end] == '+' || chars[end] == '-') && end > i && chars[end-1] != 'e' && chars[end-1] != 'E' { break; }
                end += 1;
            }
            if end > i && end <= chars.len() {
                let n: String = chars[i..end].iter().collect();
                if !n.is_empty() && n != "-" && !n.chars().all(|c| c == '.' || c == '+' || c == '-') {
                    result.push_str(&format!("\x1b[94m{}\x1b[0m", n));
                    i = end;
                    continue;
                }
            }
        }

        if ch == '{' || ch == '}' || ch == '[' || ch == ']' || ch == '(' || ch == ')' {
            result.push_str(&format!("\x1b[93m{}\x1b[0m", ch));
            i += 1;
            continue;
        }

        if ch == 't' && output[i..].starts_with("true") {
            result.push_str("\x1b[95mtrue\x1b[0m");
            i += 4;
            continue;
        }
        if ch == 'f' && output[i..].starts_with("false") {
            result.push_str("\x1b[95mfalse\x1b[0m");
            i += 5;
            continue;
        }
        if ch == 'n' && output[i..].starts_with("null") {
            result.push_str("\x1b[95mnull\x1b[0m");
            i += 4;
            continue;
        }
        if ch == 'u' && output[i..].starts_with("undefined") {
            result.push_str("\x1b[90mundefined\x1b[0m");
            i += 9;
            continue;
        }

        result.push(ch);
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
    fn test_eval_unknown() {
        let mut shell = ShellRepl::new();
        let result = shell.eval("nonexistent_cmd").unwrap();
        assert!(result.contains("Unknown command"));
    }

    #[test]
    fn test_shell_words_split() {
        let tokens = shell_words_split("echo hello world");
        assert_eq!(tokens, vec!["echo", "hello", "world"]);

        let tokens = shell_words_split("echo 'hello world'");
        assert_eq!(tokens, vec!["echo", "hello world"]);

        let tokens = shell_words_split("echo \"hello world\"");
        assert_eq!(tokens, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_colorize_output() {
        let colored = colorize_output("hello \"world\" 42");
        assert!(colored.contains("\x1b[32m"));
        assert!(colored.contains("\x1b[94m"));
        assert!(colored.contains("\x1b[0m"));
    }

    #[test]
    fn test_colorize_keywords() {
        let colored = colorize_output("true false null undefined");
        assert!(colored.contains("\x1b[95mtrue\x1b[0m"));
        assert!(colored.contains("\x1b[95mfalse\x1b[0m"));
        assert!(colored.contains("\x1b[95mnull\x1b[0m"));
        assert!(colored.contains("\x1b[90mundefined\x1b[0m"));
    }

    #[test]
    fn test_colorize_strings() {
        let colored = colorize_output("'hello' \"world\"");
        assert!(colored.contains("\x1b[32m'hello'\x1b[0m"));
        assert!(colored.contains("\x1b[32m\"world\"\x1b[0m"));
    }

    #[test]
    fn test_colorize_numbers() {
        let colored = colorize_output("42 -3.14");
        assert!(colored.contains("\x1b[94m42\x1b[0m"));
    }

    #[test]
    fn test_global_apis_list() {
        assert!(GLOBAL_APIS.contains(&"console"));
        assert!(GLOBAL_APIS.contains(&"fetch"));
        assert!(GLOBAL_APIS.contains(&"process"));
        assert!(GLOBAL_APIS.contains(&"setTimeout"));
    }

    #[test]
    fn test_eval_mode() {
        let mut shell = ShellRepl::new();
        assert!(!shell.eval_mode());
        let shell2 = ShellRepl::new().with_eval_mode();
        assert!(shell2.eval_mode());
    }

    #[test]
    fn test_history() {
        let shell = ShellRepl::new();
        let hist = shell.history();
        assert!(hist.is_empty());
    }
}
