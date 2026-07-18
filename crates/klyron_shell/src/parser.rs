#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    String(String),
    Pipe,
    RedirectOut,
    RedirectIn,
    RedirectAppend,
    RedirectErr,
    Background,
    Semicolon,
    And,
    Or,
    Subshell(String),
}

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub stdin_redirect: Option<String>,
    pub stdout_redirect: Option<String>,
    pub stderr_redirect: Option<String>,
    pub append_stdout: bool,
    pub background: bool,
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub commands: Vec<Command>,
    pub background: bool,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub pipeline: Pipeline,
    pub stdin_redirect: Option<String>,
    pub stdout_redirect: Option<String>,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if escaped {
            current.push(ch);
            escaped = false;
            i += 1;
            continue;
        }

        if ch == '\\' && !in_single {
            escaped = true;
            i += 1;
            continue;
        }

        if ch == '\'' && !in_double {
            in_single = !in_single;
            i += 1;
            continue;
        }

        if ch == '"' && !in_single {
            in_double = !in_double;
            i += 1;
            continue;
        }

        if !in_single && !in_double {
            match ch {
                '|' => {
                    if !current.is_empty() {
                        tokens.push(Token::Word(std::mem::take(&mut current)));
                    }
                    tokens.push(Token::Pipe);
                    i += 1;
                    continue;
                }
                '>' => {
                    if !current.is_empty() {
                        tokens.push(Token::Word(std::mem::take(&mut current)));
                    }
                    if i + 1 < chars.len() && chars[i + 1] == '>' {
                        tokens.push(Token::RedirectAppend);
                        i += 2;
                    } else {
                        tokens.push(Token::RedirectOut);
                        i += 1;
                    }
                    continue;
                }
                '<' => {
                    if !current.is_empty() {
                        tokens.push(Token::Word(std::mem::take(&mut current)));
                    }
                    tokens.push(Token::RedirectIn);
                    i += 1;
                    continue;
                }
                '&' => {
                    if !current.is_empty() {
                        tokens.push(Token::Word(std::mem::take(&mut current)));
                    }
                    if i + 1 < chars.len() && chars[i + 1] == '&' {
                        tokens.push(Token::And);
                        i += 2;
                    } else {
                        tokens.push(Token::Background);
                        i += 1;
                    }
                    continue;
                }
                ';' => {
                    if !current.is_empty() {
                        tokens.push(Token::Word(std::mem::take(&mut current)));
                    }
                    tokens.push(Token::Semicolon);
                    i += 1;
                    continue;
                }
                ' ' | '\t' => {
                    if !current.is_empty() {
                        tokens.push(Token::Word(std::mem::take(&mut current)));
                    }
                    i += 1;
                    continue;
                }
                _ => {}
            }
        }

        current.push(ch);
        i += 1;
    }

    if !current.is_empty() {
        tokens.push(Token::Word(current));
    }

    tokens
}

pub fn parse(tokens: &[Token]) -> anyhow::Result<Vec<Job>> {
    let mut jobs = Vec::new();
    let mut current_pipeline = Vec::new();
    let mut current_args: Vec<String> = Vec::new();
    let mut stdin_redirect: Option<String> = None;
    let mut stdout_redirect: Option<String> = None;
    let mut stderr_redirect: Option<String> = None;
    let mut append_stdout = false;
    let mut background = false;
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Word(s) | Token::String(s) => {
                current_args.push(s.clone());
            }
            Token::Pipe => {
                if !current_args.is_empty() {
                    current_pipeline.push(Command {
                        program: current_args.remove(0),
                        args: std::mem::take(&mut current_args),
                        stdin_redirect: None,
                        stdout_redirect: None,
                        stderr_redirect: None,
                        append_stdout: false,
                        background: false,
                    });
                }
            }
            Token::RedirectOut | Token::RedirectAppend => {
                append_stdout = matches!(tokens[i], Token::RedirectAppend);
                if i + 1 < tokens.len() {
                    if let Token::Word(path) | Token::String(path) = &tokens[i + 1] {
                        stdout_redirect = Some(path.clone());
                        i += 1;
                    }
                }
            }
            Token::RedirectIn => {
                if i + 1 < tokens.len() {
                    if let Token::Word(path) | Token::String(path) = &tokens[i + 1] {
                        stdin_redirect = Some(path.clone());
                        i += 1;
                    }
                }
            }
            Token::RedirectErr => {
                if i + 1 < tokens.len() {
                    if let Token::Word(path) | Token::String(path) = &tokens[i + 1] {
                        stderr_redirect = Some(path.clone());
                        i += 1;
                    }
                }
            }
            Token::Background => {
                background = true;
            }
            Token::Semicolon | Token::And | Token::Or => {
                if !current_args.is_empty() {
                    current_pipeline.push(Command {
                        program: current_args.remove(0),
                        args: std::mem::take(&mut current_args),
                        stdin_redirect: stdin_redirect.take(),
                        stdout_redirect: stdout_redirect.take(),
                        stderr_redirect: stderr_redirect.take(),
                        append_stdout,
                        background,
                    });
                }
                if !current_pipeline.is_empty() {
                    jobs.push(Job {
                        pipeline: Pipeline {
                            commands: std::mem::take(&mut current_pipeline),
                            background,
                        },
                        stdin_redirect: stdin_redirect.take(),
                        stdout_redirect: stdout_redirect.take(),
                    });
                }
                append_stdout = false;
                background = false;
            }
            Token::Subshell(s) => {
                current_args.push(s.clone());
            }
        }
        i += 1;
    }

    if !current_args.is_empty() {
        current_pipeline.push(Command {
            program: current_args.remove(0),
            args: std::mem::take(&mut current_args),
            stdin_redirect: stdin_redirect.take(),
            stdout_redirect: stdout_redirect.take(),
            stderr_redirect: stderr_redirect.take(),
            append_stdout,
            background,
        });
    }

    if !current_pipeline.is_empty() {
        jobs.push(Job {
            pipeline: Pipeline {
                commands: std::mem::take(&mut current_pipeline),
                background,
            },
            stdin_redirect: stdin_redirect.take(),
            stdout_redirect: stdout_redirect.take(),
        });
    }

    Ok(jobs)
}

pub fn parse_line(input: &str) -> anyhow::Result<Vec<Job>> {
    let tokens = tokenize(input);
    parse(&tokens)
}

pub fn shell_words_split(input: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("echo hello world");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Word("echo".into()));
        assert_eq!(tokens[1], Token::Word("hello".into()));
        assert_eq!(tokens[2], Token::Word("world".into()));
    }

    #[test]
    fn test_tokenize_pipe() {
        let tokens = tokenize("echo hello | wc -c");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[2], Token::Pipe);
    }

    #[test]
    fn test_tokenize_redirect() {
        let tokens = tokenize("cat < input > output");
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[1], Token::RedirectIn));
        assert!(matches!(tokens[3], Token::RedirectOut));
    }

    #[test]
    fn test_tokenize_background() {
        let tokens = tokenize("sleep 10 &");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[2], Token::Background);
    }

    #[test]
    fn test_tokenize_strings() {
        let tokens = tokenize("echo 'hello world'");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[1], Token::Word("hello world".into()));
    }

    #[test]
    fn test_parse_simple() {
        let jobs = parse_line("echo hello").unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].pipeline.commands[0].program, "echo");
        assert_eq!(jobs[0].pipeline.commands[0].args, vec!["hello"]);
    }

    #[test]
    fn test_parse_pipeline() {
        let jobs = parse_line("echo hello | wc -c").unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].pipeline.commands.len(), 2);
        assert_eq!(jobs[0].pipeline.commands[0].program, "echo");
        assert_eq!(jobs[0].pipeline.commands[1].program, "wc");
    }

    #[test]
    fn test_parse_with_redirects() {
        let jobs = parse_line("cat < input.txt > output.txt").unwrap();
        assert_eq!(jobs.len(), 1);
        let cmd = &jobs[0].pipeline.commands[0];
        assert_eq!(cmd.program, "cat");
        assert_eq!(cmd.stdin_redirect, Some("input.txt".into()));
        assert_eq!(cmd.stdout_redirect, Some("output.txt".into()));
    }

    #[test]
    fn test_parse_background() {
        let jobs = parse_line("sleep 10 &").unwrap();
        assert!(jobs[0].pipeline.background);
    }

    #[test]
    fn test_shell_words_split() {
        let tokens = shell_words_split("echo 'hello world'");
        assert_eq!(tokens, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_tokenize_redirect_append() {
        let tokens = tokenize("echo hello >> log.txt");
        assert!(matches!(tokens[2], Token::RedirectAppend));
    }

    #[test]
    fn test_parse_semicolon() {
        let jobs = parse_line("echo a; echo b").unwrap();
        assert_eq!(jobs.len(), 2);
    }
}
