use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use similar::TextDiff;

use crate::config::{Cache, DiffChange, FormatDiff, FormatReport, FormatterConfig, IndentStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormatBackend {
    Prettier,
    Biome,
    Rustfmt,
    Gofmt,
    Black,
    Rubocop,
    Pint,
}

impl FormatBackend {
    pub fn name(self) -> &'static str {
        match self {
            FormatBackend::Prettier => "Prettier",
            FormatBackend::Biome => "Biome",
            FormatBackend::Rustfmt => "rustfmt",
            FormatBackend::Gofmt => "gofmt",
            FormatBackend::Black => "Black",
            FormatBackend::Rubocop => "RuboCop",
            FormatBackend::Pint => "Pint",
        }
    }

    pub fn command(self) -> (&'static str, Vec<&'static str>) {
        match self {
            FormatBackend::Prettier => ("npx", vec!["prettier", "--check", "."]),
            FormatBackend::Biome => ("npx", vec!["biome", "format"]),
            FormatBackend::Rustfmt => ("cargo", vec!["fmt", "--check"]),
            FormatBackend::Gofmt => ("gofmt", vec!["-l", "."]),
            FormatBackend::Black => ("black", vec!["--check", "."]),
            FormatBackend::Rubocop => ("rubocop", vec!["-a", "--only", "Layout"]),
            FormatBackend::Pint => ("./vendor/bin/pint", vec!["--test"]),
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            FormatBackend::Prettier => &["js", "jsx", "ts", "tsx", "json", "css", "md", "html", "yaml", "yml"],
            FormatBackend::Biome => &["js", "jsx", "ts", "tsx", "json"],
            FormatBackend::Rustfmt => &["rs"],
            FormatBackend::Gofmt => &["go"],
            FormatBackend::Black => &["py"],
            FormatBackend::Rubocop => &["rb"],
            FormatBackend::Pint => &["php"],
        }
    }

    pub fn is_jsts(self) -> bool {
        matches!(self, FormatBackend::Prettier | FormatBackend::Biome)
    }
}

#[derive(Debug, Default)]
pub struct Formatter {
    config: FormatterConfig,
    cache: Cache,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter::default()
    }

    pub fn with_config(config: FormatterConfig) -> Self {
        Formatter {
            config,
            cache: Cache::new(),
        }
    }

    pub fn config(&self) -> &FormatterConfig {
        &self.config
    }

    pub fn set_indent(&mut self, size: u8, style: IndentStyle) {
        self.config.indent_size = size;
        self.config.indent_style = style;
    }

    pub fn set_line_width(&mut self, width: u16) {
        self.config.line_width = width;
    }

    pub fn detect(dir: &Path) -> FormatBackend {
        if Self::has_prettier_config(dir) {
            FormatBackend::Prettier
        } else if dir.join("biome.json").exists() {
            FormatBackend::Biome
        } else if dir.join("Cargo.toml").exists() {
            FormatBackend::Rustfmt
        } else if dir.join("go.mod").exists() {
            FormatBackend::Gofmt
        } else if dir.join("pyproject.toml").exists() {
            FormatBackend::Black
        } else if dir.join(".rubocop.yml").exists() || dir.join(".rubocop.yaml").exists() {
            FormatBackend::Rubocop
        } else if dir.join("pint.json").exists() {
            FormatBackend::Pint
        } else {
            FormatBackend::Prettier
        }
    }

    pub fn detect_with_auto_discovery(dir: &Path) -> FormatBackend {
        let mut current = Some(dir);
        while let Some(d) = current {
            if Self::has_prettier_config(d) {
                return FormatBackend::Prettier;
            }
            if d.join("biome.json").exists() {
                return FormatBackend::Biome;
            }
            if d.join(".rubocop.yml").exists() || d.join(".rubocop.yaml").exists() {
                return FormatBackend::Rubocop;
            }
            if d.join("pint.json").exists() {
                return FormatBackend::Pint;
            }
            current = d.parent();
        }
        Self::detect(dir)
    }

    fn has_prettier_config(dir: &Path) -> bool {
        dir.join(".prettierrc").exists()
            || dir.join(".prettierrc.json").exists()
            || dir.join(".prettierrc.yaml").exists()
            || dir.join(".prettierrc.yml").exists()
            || dir.join(".prettierrc.js").exists()
            || dir.join("prettier.config.js").exists()
    }

    fn load_prettier_ignore(dir: &Path) -> Gitignore {
        let mut builder = GitignoreBuilder::new(dir);
        let ignore_path = dir.join(".prettierignore");
        if ignore_path.exists() {
            let _ = builder.add(ignore_path);
        }
        builder.build().unwrap_or(Gitignore::empty())
    }

    pub(crate) fn content_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn format_content_inline(&self, content: &str, language: &str) -> Result<String> {
        match language {
            "js" | "jsx" | "ts" | "tsx" | "json" => self.format_js_like(content, language),
            _ => Ok(content.to_string()),
        }
    }

    fn format_js_like(&self, content: &str, _language: &str) -> Result<String> {
        let indent = match self.config.indent_style {
            IndentStyle::Spaces => " ".repeat(self.config.indent_size as usize),
            IndentStyle::Tabs => "\t".to_string(),
        };

        let mut result = String::new();
        let mut depth: isize = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
                depth -= 1;
                let d = depth.max(0) as usize;
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&indent.repeat(d));
                result.push_str(trimmed);
            } else if trimmed.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                continue;
            } else {
                if !result.is_empty() {
                    result.push('\n');
                }
                let line_depth = depth.max(0) as usize;
                result.push_str(&indent.repeat(line_depth));
                result.push_str(trimmed);
            }

            let open_count = trimmed.matches('{').count() as isize;
            let close_count = trimmed.matches('}').count() as isize;
            depth += open_count - close_count;
        }

        if !content.ends_with('\n') {
            result.push('\n');
        }

        Ok(result)
    }

    pub fn format_check(dir: &Path) -> Result<FormatReport> {
        Formatter::new().format_check_internal(dir)
    }

    fn format_check_internal(&self, dir: &Path) -> Result<FormatReport> {
        let backend = Self::detect(dir);
        self.run_formatter(dir, &backend, false)
    }

    pub fn format_path(&self, dir: &Path, path: &str) -> Result<FormatReport> {
        let backend = Self::detect(dir);
        let (program, _) = backend.command();
        let args = match backend {
            FormatBackend::Prettier => vec!["prettier", "--write", path],
            FormatBackend::Biome => vec!["biome", "format", "--write", path],
            FormatBackend::Rustfmt => vec!["fmt", path],
            FormatBackend::Gofmt => vec!["-w", path],
            FormatBackend::Black => vec!["black", path],
            FormatBackend::Rubocop => vec!["-a", "--only", "Layout", path],
            FormatBackend::Pint => vec![path],
        };
        let output = std::process::Command::new(program)
            .args(&args)
            .current_dir(dir)
            .output()
            .with_context(|| format!("failed to format path {path}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{stdout}\n{stderr}");
        Ok(FormatReport {
            files_changed: 1,
            files_unchanged: 0,
            files_skipped: 0,
            diffs: vec![],
            output: combined,
        })
    }

    pub fn format_write(dir: &Path) -> Result<FormatReport> {
        Formatter::new().format_write_internal(dir)
    }

    fn format_write_internal(&self, dir: &Path) -> Result<FormatReport> {
        let backend = Self::detect(dir);
        self.run_formatter(dir, &backend, true)
    }

    pub fn format_stdin(&self, dir: &Path) -> Result<FormatReport> {
        let backend = Self::detect(dir);
        let (program, _) = backend.command();

        let mut stdin_content = String::new();
        std::io::stdin()
            .read_to_string(&mut stdin_content)
            .context("failed to read stdin")?;

        let args = match backend {
            FormatBackend::Prettier => vec!["prettier", "--stdin-filepath", "stdin"],
            FormatBackend::Biome => vec!["biome", "format", "--stdin-file-path", "stdin"],
            FormatBackend::Rustfmt => {
                let mut child = std::process::Command::new("rustfmt")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .context("failed to spawn rustfmt")?;
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(stdin_content.as_bytes())?;
                }
                let output = child.wait_with_output()?;
                let formatted = String::from_utf8_lossy(&output.stdout).to_string();
                let diff = TextDiff::from_lines(&stdin_content, &formatted);
                let changes: Vec<DiffChange> = diff
                    .iter_all_changes()
                    .map(|c| DiffChange {
                        tag: format!("{:?}", c.tag()),
                        old_line: c.old_index().map(|i| i as u64 + 1),
                        new_line: c.new_index().map(|i| i as u64 + 1),
                        content: c.value().to_string(),
                    })
                    .collect();

                return Ok(FormatReport {
                    files_changed: if changes.iter().any(|c| c.tag == "Delete" || c.tag == "Insert")
                    {
                        1
                    } else {
                        0
                    },
                    files_unchanged: 0,
                    files_skipped: 0,
                    diffs: vec![FormatDiff {
                        file: "stdin".into(),
                        changes,
                    }],
                    output: formatted,
                });
            }
            _ => vec![],
        };

        let mut child = std::process::Command::new(program)
            .args(&args)
            .current_dir(dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to run {} on stdin", backend.name()))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(stdin_content.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        let formatted = String::from_utf8_lossy(&output.stdout).to_string();

        let diff = TextDiff::from_lines(&stdin_content, &formatted);
        let changes: Vec<DiffChange> = diff
            .iter_all_changes()
            .map(|c| DiffChange {
                tag: format!("{:?}", c.tag()),
                old_line: c.old_index().map(|i| i as u64 + 1),
                new_line: c.new_index().map(|i| i as u64 + 1),
                content: c.value().to_string(),
            })
            .collect();

        Ok(FormatReport {
            files_changed: if changes.iter().any(|c| c.tag == "Delete" || c.tag == "Insert") {
                1
            } else {
                0
            },
            files_unchanged: 0,
            files_skipped: 0,
            diffs: vec![FormatDiff {
                file: "stdin".into(),
                changes,
            }],
            output: formatted,
        })
    }

    pub fn format_diff(&self, dir: &Path) -> Result<FormatReport> {
        let backend = Self::detect(dir);
        let ignore = Self::load_prettier_ignore(dir);
        let extensions = backend.extensions();
        let mut report = FormatReport {
            files_changed: 0,
            files_unchanged: 0,
            files_skipped: 0,
            diffs: vec![],
            output: String::new(),
        };

        let entries = match dir.read_dir() {
            Ok(e) => e,
            Err(_) => return Ok(report),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n != "node_modules" && n != ".git" && n != "target")
                {
                    let sub = self.format_diff(&path)?;
                    report.files_changed += sub.files_changed;
                    report.files_unchanged += sub.files_unchanged;
                    report.files_skipped += sub.files_skipped;
                    report.diffs.extend(sub.diffs);
                }
                continue;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !extensions.contains(&ext) {
                    report.files_skipped += 1;
                    continue;
                }
            }

            if ignore.matched(&path, false).is_ignore() {
                report.files_skipped += 1;
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => {
                    report.files_skipped += 1;
                    continue;
                }
            };

            let content_hash = if self.config.use_cache {
                let h = Self::content_hash(&content);
                if let Ok(cache) = self.cache.map.lock() {
                    if cache.get(&path) == Some(&h) {
                        report.files_unchanged += 1;
                        continue;
                    }
                }
                Some(h)
            } else {
                None
            };

            let formatted = self.format_content(&content, backend, dir)?;
            if formatted != content {
                let diff = TextDiff::from_lines(&content, &formatted);
                let changes: Vec<DiffChange> = diff
                    .iter_all_changes()
                    .map(|c| DiffChange {
                        tag: format!("{:?}", c.tag()),
                        old_line: c.old_index().map(|i| i as u64 + 1),
                        new_line: c.new_index().map(|i| i as u64 + 1),
                        content: c.value().to_string(),
                    })
                    .collect();
                report.diffs.push(FormatDiff {
                    file: path.to_string_lossy().to_string(),
                    changes,
                });
                report.files_changed += 1;

                if self.config.write {
                    std::fs::write(&path, &formatted).context("failed to write formatted file")?;
                }

                if let Some(h) = content_hash {
                    if let Ok(mut cache) = self.cache.map.lock() {
                        cache.insert(path, h);
                    }
                }
            } else {
                report.files_unchanged += 1;
            }
        }

        Ok(report)
    }

    fn format_content(&self, content: &str, backend: FormatBackend, dir: &Path) -> Result<String> {
        if backend == FormatBackend::Rustfmt {
            let mut child = std::process::Command::new("rustfmt")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .context("failed to spawn rustfmt")?;
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(content.as_bytes())?;
            }
            let output = child.wait_with_output()?;
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }

        let (program, base_args) = backend.command();
        let mut args: Vec<&str> = base_args.iter().copied().collect();

        if backend == FormatBackend::Prettier {
            args = vec!["prettier", "--stdin-filepath", "file.js"];
        }

        let mut child = std::process::Command::new(program)
            .args(&args)
            .current_dir(dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to run {}", backend.name()))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(content.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn run_formatter(
        &self,
        dir: &Path,
        _backend: &FormatBackend,
        write: bool,
    ) -> Result<FormatReport> {
        if write {
            let mut config = self.config.clone();
            config.write = true;
            let formatter = Formatter {
                config,
                cache: Cache::new(),
            };
            formatter.format_diff(dir)
        } else {
            let mut config = self.config.clone();
            config.write = false;
            let formatter = Formatter {
                config,
                cache: Cache::new(),
            };
            formatter.format_diff(dir)
        }
    }
}
