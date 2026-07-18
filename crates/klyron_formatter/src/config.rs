use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatDiff {
    pub file: String,
    pub changes: Vec<DiffChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub tag: String,
    pub old_line: Option<u64>,
    pub new_line: Option<u64>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatReport {
    pub files_changed: u64,
    pub files_unchanged: u64,
    pub files_skipped: u64,
    pub diffs: Vec<FormatDiff>,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct FormatterConfig {
    pub write: bool,
    pub incremental: bool,
    pub use_cache: bool,
    pub indent_size: u8,
    pub indent_style: IndentStyle,
    pub line_width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

impl Default for IndentStyle {
    fn default() -> Self {
        Self::Spaces
    }
}

impl Default for FormatterConfig {
    fn default() -> Self {
        FormatterConfig {
            write: false,
            incremental: true,
            use_cache: true,
            indent_size: 2,
            indent_style: IndentStyle::Spaces,
            line_width: 80,
        }
    }
}

#[derive(Default, Debug)]
pub struct Cache {
    pub map: Mutex<HashMap<PathBuf, String>>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            map: Mutex::new(HashMap::new()),
        }
    }
}
