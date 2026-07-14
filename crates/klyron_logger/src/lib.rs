use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

#[derive(Debug, Serialize)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
}

pub struct Logger {
    min_level: LogLevel,
    json_output: bool,
    file_output: Option<Mutex<std::fs::File>>,
}

impl Logger {
    pub fn new() -> Self {
        Self { min_level: LogLevel::Info, json_output: false, file_output: None }
    }

    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    pub fn with_json(mut self, enabled: bool) -> Self {
        self.json_output = enabled;
        self
    }

    pub fn with_file(mut self, path: &Path) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        self.file_output = Some(Mutex::new(file));
        Ok(self)
    }

    pub fn debug(&self, msg: &str) { self.log(LogLevel::Debug, msg, None); }
    pub fn info(&self, msg: &str) { self.log(LogLevel::Info, msg, None); }
    pub fn warn(&self, msg: &str) { self.log(LogLevel::Warn, msg, None); }
    pub fn error(&self, msg: &str) { self.log(LogLevel::Error, msg, None); }

    pub fn debug_with(&self, msg: &str, meta: serde_json::Value) { self.log(LogLevel::Debug, msg, Some(meta)); }
    pub fn info_with(&self, msg: &str, meta: serde_json::Value) { self.log(LogLevel::Info, msg, Some(meta)); }
    pub fn warn_with(&self, msg: &str, meta: serde_json::Value) { self.log(LogLevel::Warn, msg, Some(meta)); }
    pub fn error_with(&self, msg: &str, meta: serde_json::Value) { self.log(LogLevel::Error, msg, Some(meta)); }

    fn log(&self, level: LogLevel, msg: &str, metadata: Option<serde_json::Value>) {
        if (level as u8) < (self.min_level as u8) { return; }

        let entry = LogEntry {
            timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            level: level.as_str().to_string(),
            message: msg.to_string(),
            module: None,
            file: None,
            line: None,
            metadata,
        };

        if self.json_output {
            let json = serde_json::to_string(&entry).unwrap_or_default();
            self.write_output(&json);
        } else {
            let ts = &entry.timestamp[..23];
            let line = format!("{} [{}] {}", ts, entry.level, entry.message);
            self.write_output(&line);
        }
    }

    fn write_output(&self, line: &str) {
        match self.min_level {
            LogLevel::Error => eprintln!("{line}"),
            _ => println!("{line}"),
        }

        if let Some(ref file) = self.file_output {
            if let Ok(mut f) = file.lock() {
                let _ = writeln!(f, "{line}");
                let _ = f.flush();
            }
        }
    }
}

impl Default for Logger {
    fn default() -> Self { Self::new() }
}

pub fn info(msg: &str) {
    Logger::new().info(msg);
}

pub fn warn(msg: &str) {
    Logger::new().warn(msg);
}

pub fn error(msg: &str) {
    Logger::new().error(msg);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_logger_new() {
        let logger = Logger::new();
        logger.info("test message");
    }
}
