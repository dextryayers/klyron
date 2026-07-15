use std::fmt;
use serde::Serialize;
use crate::colors::{Color, style_success, style_error, style_warning, style_info, supports_color};
use crate::progress::ProgressBar;
use crate::error::KlyronError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    JsonPretty,
    Silent,
}

impl OutputFormat {
    pub fn from_flags(json: bool, quiet: bool) -> Self {
        if quiet { return OutputFormat::Silent; }
        if json { return OutputFormat::JsonPretty; }
        OutputFormat::Text
    }

    pub fn is_json(&self) -> bool {
        matches!(self, OutputFormat::Json | OutputFormat::JsonPretty)
    }

    pub fn is_silent(&self) -> bool {
        *self == OutputFormat::Silent
    }
}

#[derive(Debug, Clone)]
pub struct OutputOptions {
    pub format: OutputFormat,
    pub color: bool,
    pub verbose: bool,
    pub quiet: bool,
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            format: OutputFormat::Text,
            color: supports_color(),
            verbose: false,
            quiet: false,
        }
    }
}

impl OutputOptions {
    pub fn new(format: OutputFormat, color: bool, verbose: bool, quiet: bool) -> Self {
        Self { format, color, verbose, quiet }
    }
}

pub trait OutputFormatter: Send + Sync {
    fn format<T: Serialize>(&self, value: &T, options: &OutputOptions) -> Result<String, KlyronError>;
    fn print<T: Serialize>(&self, value: &T, options: &OutputOptions) -> Result<(), KlyronError> {
        let output = self.format(value, options)?;
        if !options.format.is_silent() {
            println!("{}", output);
        }
        Ok(())
    }
    fn name(&self) -> &'static str;
}

pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format<T: Serialize>(&self, value: &T, options: &OutputOptions) -> Result<String, KlyronError> {
        match options.format {
            OutputFormat::JsonPretty => {
                serde_json::to_string_pretty(value).map_err(|e| KlyronError::InternalError(format!("JSON serialization: {e}")))
            }
            _ => {
                serde_json::to_string(value).map_err(|e| KlyronError::InternalError(format!("JSON serialization: {e}")))
            }
        }
    }

    fn name(&self) -> &'static str { "json" }
}

pub struct TableFormatter {
    pub headers: Vec<String>,
    pub widths: Vec<usize>,
}

impl TableFormatter {
    pub fn new(headers: Vec<String>) -> Self {
        let widths = headers.iter().map(|h| h.len()).collect();
        Self { headers, widths }
    }

    pub fn add_row(&mut self, row: &[String]) {
        for (i, cell) in row.iter().enumerate() {
            if i < self.widths.len() {
                self.widths[i] = self.widths[i].max(cell.len());
            }
        }
    }

    pub fn render(&self, rows: &[Vec<String>]) -> String {
        if self.headers.is_empty() && rows.is_empty() {
            return String::new();
        }
        let mut out = String::new();

        if !self.headers.is_empty() {
            let header_line: String = self.headers.iter().enumerate()
                .map(|(i, h)| format!(" {:<width$}", h, width = self.widths.get(i).copied().unwrap_or(h.len())))
                .collect::<Vec<_>>()
                .join(" |");
            out.push_str(&header_line);
            out.push('\n');

            let sep: String = self.widths.iter()
                .map(|w| "-".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("|");
            out.push_str(&sep);
            out.push('\n');
        }

        for row in rows {
            let line: String = row.iter().enumerate()
                .map(|(i, cell)| format!(" {:<width$}", cell, width = self.widths.get(i).copied().unwrap_or(cell.len())))
                .collect::<Vec<_>>()
                .join(" |");
            out.push_str(&line);
            out.push('\n');
        }

        out
    }
}

impl OutputFormatter for TableFormatter {
    fn format<T: Serialize>(&self, value: &T, _options: &OutputOptions) -> Result<String, KlyronError> {
        // For non-list types, fall back to JSON
        JsonFormatter.format(value, &OutputOptions::default())
    }

    fn name(&self) -> &'static str { "table" }
}

pub struct ColorFormatter;

impl OutputFormatter for ColorFormatter {
    fn format<T: Serialize>(&self, value: &T, options: &OutputOptions) -> Result<String, KlyronError> {
        if options.color {
            let json = JsonFormatter.format(value, options)?;
            Ok(syntax_highlight_json(&json))
        } else {
            JsonFormatter.format(value, options)
        }
    }

    fn name(&self) -> &'static str { "color" }
}

fn syntax_highlight_json(json: &str) -> String {
    let mut out = String::new();
    let mut chars = json.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                let mut key = String::from('"');
                loop {
                    match chars.next() {
                        Some('\\') => { key.push('\\'); if let Some(esc) = chars.next() { key.push(esc); } }
                        Some('"') => { key.push('"'); break; }
                        Some(ch) => key.push(ch),
                        None => break,
                    }
                }
                let is_key = chars.peek() == Some(&':');
                if is_key {
                    out.push_str(&Color::CYAN.paint(&key));
                } else {
                    out.push_str(&Color::GREEN.paint(&key));
                }
            }
            ':' => { out.push_str(&Color::DIM.paint(": ")); chars.next(); }
            '{' | '[' => { out.push(c); }
            '}' | ']' => { out.push(c); }
            ',' => { out.push_str(&Color::DIM.paint(", ")); }
            c if c.is_ascii_digit() || c == '-' => {
                let mut num = String::from(c);
                while let Some(n) = chars.peek() {
                    if n.is_ascii_digit() || *n == '.' || *n == 'e' || *n == 'E' || *n == '+' || *n == '-' {
                        num.push(chars.next().unwrap());
                    } else { break; }
                }
                out.push_str(&Color::MAGENTA.paint(&num));
            }
            't' | 'f' | 'n' => {
                let mut word = String::from(c);
                for _ in 0..3 {
                    if let Some(n) = chars.next() { word.push(n); }
                }
                out.push_str(&Color::YELLOW.paint(&word));
            }
            _ => out.push(c),
        }
    }
    out
}

pub struct ProgressReporter {
    bars: Vec<ProgressBar>,
    format: OutputFormat,
}

impl ProgressReporter {
    pub fn new(format: OutputFormat) -> Self {
        Self { bars: Vec::new(), format }
    }

    pub fn add_bar(&mut self, total: u64, message: &str) -> usize {
        let idx = self.bars.len();
        self.bars.push(ProgressBar::new(total, message));
        idx
    }

    pub fn tick(&mut self, idx: usize, n: u64) {
        if self.format.is_silent() { return; }
        if let Some(bar) = self.bars.get_mut(idx) {
            bar.tick(n);
        }
    }

    pub fn set_message(&mut self, idx: usize, msg: &str) {
        if self.format.is_silent() { return; }
        if let Some(bar) = self.bars.get_mut(idx) {
            bar.set_message(msg);
        }
    }

    pub fn finish(&mut self, idx: usize) {
        if let Some(bar) = self.bars.get_mut(idx) {
            bar.finish();
        }
    }

    pub fn finish_all(&mut self) {
        for i in 0..self.bars.len() {
            self.finish(i);
        }
    }
}

pub fn format_result<T: Serialize, F: OutputFormatter>(formatter: &F, value: &T, options: &OutputOptions) -> Result<(), KlyronError> {
    formatter.print(value, options)
}

pub fn format_success(msg: &str, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            serde_json::json!({"status": "ok", "message": msg}).to_string()
        }
        OutputFormat::Silent => String::new(),
        OutputFormat::Text => style_success(msg),
    }
}

pub fn format_error(msg: &str, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json | OutputFormat::JsonPretty => {
            serde_json::json!({"status": "error", "message": msg}).to_string()
        }
        OutputFormat::Silent => String::new(),
        OutputFormat::Text => style_error(msg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_flags() {
        assert_eq!(OutputFormat::from_flags(false, false), OutputFormat::Text);
        assert_eq!(OutputFormat::from_flags(true, false), OutputFormat::JsonPretty);
        assert_eq!(OutputFormat::from_flags(false, true), OutputFormat::Silent);
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter;
        let data = serde_json::json!({"name": "test", "value": 42});
        let opts = OutputOptions { format: OutputFormat::JsonPretty, ..Default::default() };
        let result = formatter.format(&data, &opts).unwrap();
        assert!(result.contains("test"));
    }

    #[test]
    fn test_json_formatter_compact() {
        let formatter = JsonFormatter;
        let data = serde_json::json!({"name": "test"});
        let opts = OutputOptions { format: OutputFormat::Json, ..Default::default() };
        let result = formatter.format(&data, &opts).unwrap();
        assert!(result.contains("test"));
    }

    #[test]
    fn test_table_formatter() {
        let mut table = TableFormatter::new(vec!["Name".into(), "Value".into()]);
        table.add_row(&["foo".into(), "42".into()]);
        let output = table.render(&[vec!["foo".into(), "42".into()]]);
        assert!(output.contains("foo"));
        assert!(output.contains("Name"));
    }

    #[test]
    fn test_syntax_highlight() {
        let highlighted = syntax_highlight_json(r#"{"key": "value", "num": 42}"#);
        assert!(highlighted.contains("key"));
        assert!(highlighted.contains("value"));
    }

    #[test]
    fn test_format_success() {
        let s = format_success("done", OutputFormat::Text);
        assert!(s.contains("done"));
    }

    #[test]
    fn test_format_error() {
        let s = format_error("failed", OutputFormat::Text);
        assert!(s.contains("failed"));
    }

    #[test]
    fn test_format_success_json() {
        let s = format_success("done", OutputFormat::Json);
        assert!(s.contains("ok"));
    }

    #[test]
    fn test_progress_reporter() {
        let mut reporter = ProgressReporter::new(OutputFormat::Text);
        let idx = reporter.add_bar(100, "testing");
        reporter.tick(idx, 50);
        reporter.finish(idx);
    }
}
