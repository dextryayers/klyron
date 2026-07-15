use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TRACE" => Some(Self::Trace),
            "DEBUG" => Some(Self::Debug),
            "INFO" => Some(Self::Info),
            "WARN" | "WARNING" => Some(Self::Warn),
            "ERROR" => Some(Self::Error),
            "FATAL" => Some(Self::Fatal),
            _ => None,
        }
    }

    pub fn num_level(&self) -> u8 {
        match self {
            Self::Trace => 1,
            Self::Debug => 2,
            Self::Info => 3,
            Self::Warn => 4,
            Self::Error => 5,
            Self::Fatal => 6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub rotation: Option<LogRotationConfig>,
    pub fields: HashMap<String, String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::Stdout,
            rotation: None,
            fields: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File(PathBuf),
    Syslog(String),
    Tcp(String),
    Udp(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    pub max_size_mb: u64,
    pub max_age_days: u64,
    pub max_backups: u32,
    pub compress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub module: String,
    pub file: String,
    pub line: u32,
    pub fields: HashMap<String, String>,
}

pub struct StructuredLogger {
    config: LogConfig,
    writer: Option<Mutex<File>>,
    current_size: Mutex<u64>,
}

impl StructuredLogger {
    pub fn new(config: LogConfig) -> Self {
        let writer = match &config.output {
            LogOutput::File(path) => {
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .ok()
                    .map(Mutex::new)
            }
            _ => None,
        };
        Self {
            config,
            writer,
            current_size: Mutex::new(0),
        }
    }

    pub fn log(&self, level: LogLevel, message: &str, module: &str, file: &str, line: u32, fields: HashMap<String, String>) {
        if level.num_level() < self.config.level.num_level() {
            return;
        }
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: level.as_str().to_string(),
            message: message.to_string(),
            module: module.to_string(),
            file: file.to_string(),
            line,
            fields,
        };
        let formatted = match self.config.format {
            LogFormat::Json => serde_json::to_string(&entry).unwrap_or_default(),
            LogFormat::Text => {
                let f = if entry.fields.is_empty() {
                    String::new()
                } else {
                    format!(" {}", serde_json::to_string(&entry.fields).unwrap_or_default())
                };
                format!("{} [{}] {} ({}){f}", entry.timestamp, entry.level, entry.message, entry.module)
            }
        };
        match &self.config.output {
            LogOutput::Stdout => {
                println!("{formatted}");
            }
            LogOutput::Stderr => {
                eprintln!("{formatted}");
            }
            LogOutput::File(_) => {
                if let Some(ref writer) = self.writer {
                    if let Ok(mut w) = writer.lock() {
                        let _ = writeln!(w, "{formatted}");
                        let _ = w.flush();
                    }
                }
            }
            LogOutput::Syslog(_addr) => {
                println!("syslog>{formatted}");
            }
            LogOutput::Tcp(_addr) => {
                println!("tcp>{formatted}");
            }
            LogOutput::Udp(_addr) => {
                println!("udp>{formatted}");
            }
        }
        if let Some(ref rotation) = self.config.rotation {
            self.check_rotation(rotation);
        }
    }

    fn check_rotation(&self, rotation: &LogRotationConfig) {
        if let Ok(mut size) = self.current_size.lock() {
            if *size >= rotation.max_size_mb * 1024 * 1024 {
                if let LogOutput::File(ref path) = self.config.output {
                    let rotated_path = path.with_extension(format!("{}.old", path.extension().and_then(|e| e.to_str()).unwrap_or("log")));
                    let _ = fs::rename(path, &rotated_path);
                    *size = 0;
                    if let Some(parent) = path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if let Ok(new_file) = OpenOptions::new().create(true).append(true).open(path) {
                        if let Some(ref writer) = self.writer {
                            if let Ok(mut w) = writer.lock() {
                                *w = new_file;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message, "", "", 0, HashMap::new());
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message, "", "", 0, HashMap::new());
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message, "", "", 0, HashMap::new());
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message, "", "", 0, HashMap::new());
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message, "", "", 0, HashMap::new());
    }

    pub fn fatal(&self, message: &str) {
        self.log(LogLevel::Fatal, message, "", "", 0, HashMap::new());
    }
}

pub struct LoggerBuilder {
    config: LogConfig,
}

impl LoggerBuilder {
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
        }
    }

    pub fn level(mut self, level: LogLevel) -> Self {
        self.config.level = level;
        self
    }

    pub fn format(mut self, format: LogFormat) -> Self {
        self.config.format = format;
        self
    }

    pub fn output(mut self, output: LogOutput) -> Self {
        self.config.output = output;
        self
    }

    pub fn rotation(mut self, rotation: LogRotationConfig) -> Self {
        self.config.rotation = Some(rotation);
        self
    }

    pub fn field(mut self, key: &str, value: &str) -> Self {
        self.config.fields.insert(key.to_string(), value.to_string());
        self
    }

    pub fn build(self) -> StructuredLogger {
        StructuredLogger::new(self.config)
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LogRotation;

impl LogRotation {
    pub fn size_based(max_size_mb: u64, max_backups: u32) -> LogRotationConfig {
        LogRotationConfig {
            max_size_mb,
            max_age_days: 0,
            max_backups,
            compress: false,
        }
    }

    pub fn time_based(max_age_days: u64) -> LogRotationConfig {
        LogRotationConfig {
            max_size_mb: 0,
            max_age_days,
            max_backups: 0,
            compress: true,
        }
    }

    pub fn combined(max_size_mb: u64, max_age_days: u64, max_backups: u32) -> LogRotationConfig {
        LogRotationConfig {
            max_size_mb,
            max_age_days,
            max_backups,
            compress: true,
        }
    }
}
