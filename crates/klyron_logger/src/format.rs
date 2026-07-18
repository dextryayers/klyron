use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Trace => "\x1b[90m",
            LogLevel::Debug => "\x1b[36m",
            LogLevel::Info => "\x1b[32m",
            LogLevel::Warn => "\x1b[33m",
            LogLevel::Error => "\x1b[31m",
            LogLevel::Fatal => "\x1b[35m",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(LogLevel::Trace),
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" | "WARNING" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            "FATAL" => Some(LogLevel::Fatal),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: &str) -> Self {
        LogEntry {
            timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            level: level.as_str().to_string(),
            message: message.to_string(),
            module: None,
            file: None,
            line: None,
            fields: None,
        }
    }

    pub fn with_fields(mut self, fields: serde_json::Value) -> Self {
        self.fields = Some(fields);
        self
    }

    pub fn with_location(mut self, file: &str, line: u32) -> Self {
        self.file = Some(file.to_string());
        self.line = Some(line);
        self
    }

    pub fn with_module(mut self, module: &str) -> Self {
        self.module = Some(module.to_string());
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn to_colored_text(&self, color_enabled: bool) -> String {
        let ts = &self.timestamp[..23];
        if color_enabled {
            let color = match LogLevel::from_str(&self.level) {
                Some(level) => level.color_code(),
                None => "",
            };
            let reset = "\x1b[0m";
            format!("{color}{ts} [{:5}] {}{reset}", self.level, self.message)
        } else {
            format!("{ts} [{:5}] {}", self.level, self.message)
        }
    }

    pub fn to_text(&self) -> String {
        self.to_colored_text(false)
    }
}

#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub json_output: bool,
    pub color_enabled: bool,
    pub include_location: bool,
    pub include_module: bool,
    pub timestamp_format: TimestampFormat,
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            json_output: false,
            color_enabled: true,
            include_location: false,
            include_module: false,
            timestamp_format: TimestampFormat::Iso8601,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampFormat {
    Iso8601,
    UnixTimestamp,
    Custom(&'static str),
}

pub fn format_log_entry(entry: &LogEntry, config: &FormatConfig) -> String {
    if config.json_output {
        entry.to_json()
    } else {
        entry.to_colored_text(config.color_enabled)
    }
}
