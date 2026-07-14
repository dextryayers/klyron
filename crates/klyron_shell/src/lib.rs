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

#[derive(Clone, Debug)]
struct ShellHelper {
    commands: Vec<String>,
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
            ],
        }
    }

    fn add_command(&mut self, cmd: &str) {
        if !self.commands.contains(&cmd.to_string()) {
            self.commands.push(cmd.to_string());
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
        let line = &line[..pos];
        let mut completions = Vec::new();

        for cmd in &self.commands {
            if cmd.starts_with(line) || line.is_empty() {
                completions.push(Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                });
            }
        }

        completions.sort_by(|a, b| a.replacement.cmp(&b.replacement));
        completions.dedup_by(|a, b| a.replacement == b.replacement);

        Ok((0, completions))
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
        if line.starts_with('#') {
            Owned(format!("\x1b[90m{}\x1b[0m", line))
        } else if line.starts_with("help") || line.starts_with("exit") {
            Owned(format!("\x1b[32m{}\x1b[0m", line))
        } else if line.contains('|') {
            let colored = line
                .split("|")
                .map(|s| format!("\x1b[36m{}\x1b[0m", s.trim()))
                .collect::<Vec<_>>()
                .join(" \x1b[33m|\x1b[0m ");
            Owned(colored)
        } else {
            Borrowed(line)
        }
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _app_char: bool) -> bool {
        true
    }

    fn highlight_hint<'b>(&self, hint: &'b str) -> Cow<'b, str> {
        Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }
}

impl Validator for ShellHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for ShellHelper {}

pub struct ShellRepl {
    editor: Editor<ShellHelper, FileHistory>,
    commands: HashMap<String, Arc<dyn Fn(&[String]) -> Result<String> + Send + Sync>>,
    multiline_buffer: String,
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
            .build();

        let helper = ShellHelper::new();

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
        };

        repl.register_builtins();
        repl
    }

    fn register_builtins(&mut self) {
        self.register_command("help", |args| {
            let help_text = if args.is_empty() {
                r#"Available commands:
  help [cmd]  - Show this help or help for a specific command
  exit        - Exit the shell
  clear       - Clear the screen
  history     - Show command history
  echo        - Echo text"#
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
            Ok("Use up/down arrows to navigate history".to_string())
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

    pub fn multiline(&mut self) -> Result<String> {
        loop {
            let prompt = if self.multiline_buffer.is_empty() {
                "> "
            } else {
                ">> "
            };

            match self.editor.readline(prompt) {
                Ok(line) => {
                    if line.trim().is_empty() && !self.multiline_buffer.is_empty() {
                        let complete = std::mem::take(&mut self.multiline_buffer);
                        return Ok(complete);
                    }
                    self.multiline_buffer.push_str(&line);
                    self.multiline_buffer.push('\n');
                    let _ = self.editor.add_history_entry(line);
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    self.multiline_buffer.clear();
                    return Err(anyhow::anyhow!("Input interrupted"));
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    break;
                }
                Err(e) => return Err(anyhow::anyhow!("Readline error: {}", e)),
            }
        }
        let result = std::mem::take(&mut self.multiline_buffer);
        Ok(result)
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
            let prompt = format!("\x1b[32mklyron\x1b[0m:\x1b[34m~\x1b[0m$ ");

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let _ = self.editor.add_history_entry(line.as_str());
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
