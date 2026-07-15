use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::{DateTime, Utc};

static GLOBAL_LOGGER: once_cell::sync::Lazy<Mutex<Logger>> =
    once_cell::sync::Lazy::new(|| Mutex::new(Logger::new()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(LogLevel::Error),
            "warn" | "warning" => Some(LogLevel::Warn),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            "trace" => Some(LogLevel::Trace),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            LogLevel::Error => "\x1b[31m",
            LogLevel::Warn => "\x1b[33m",
            LogLevel::Info => "\x1b[32m",
            LogLevel::Debug => "\x1b[36m",
            LogLevel::Trace => "\x1b[90m",
        }
    }

    pub fn should_log(&self, min_level: &LogLevel) -> bool {
        *self as u8 <= *min_level as u8
    }
}

pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub module: String,
    pub file: String,
    pub line: u32,
    pub fields: HashMap<String, String>,
}

pub trait LogOutput: Send {
    fn write(&mut self, entry: LogEntry) -> std::io::Result<()>;
    fn flush(&mut self) -> std::io::Result<()>;
}

pub trait LogFormatter: Send {
    fn format(&self, entry: &LogEntry) -> String;
}

pub struct DefaultFormatter;

impl LogFormatter for DefaultFormatter {
    fn format(&self, entry: &LogEntry) -> String {
        let ts = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let level_color = entry.level.color();
        let reset = "\x1b[0m";
        format!(
            "{ts} {level_color}{:5}{reset} [{:>12}] {}",
            entry.level.as_str(),
            entry.target,
            entry.message,
        )
    }
}

pub struct JsonFormatter;

impl LogFormatter for JsonFormatter {
    fn format(&self, entry: &LogEntry) -> String {
        let mut map = serde_json::Map::new();
        map.insert("timestamp".into(), serde_json::Value::String(entry.timestamp.to_rfc3339()));
        map.insert("level".into(), serde_json::Value::String(entry.level.as_str().to_string()));
        map.insert("target".into(), serde_json::Value::String(entry.target.clone()));
        map.insert("message".into(), serde_json::Value::String(entry.message.clone()));
        map.insert("module".into(), serde_json::Value::String(entry.module.clone()));
        map.insert("file".into(), serde_json::Value::String(entry.file.clone()));
        map.insert("line".into(), serde_json::Value::Number(entry.line.into()));
        if !entry.fields.is_empty() {
            let fields: serde_json::Map<_, _> = entry.fields.iter().map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone()))).collect();
            map.insert("fields".into(), serde_json::Value::Object(fields));
        }
        serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_default()
    }
}

pub struct ConsoleOutput {
    use_color: bool,
}

impl ConsoleOutput {
    pub fn new(use_color: bool) -> Self {
        Self { use_color }
    }
}

impl LogOutput for ConsoleOutput {
    fn write(&mut self, entry: LogEntry) -> std::io::Result<()> {
        let formatter = if self.use_color {
            Box::new(DefaultFormatter) as Box<dyn LogFormatter>
        } else {
            Box::new(PlainFormatter) as Box<dyn LogFormatter>
        };
        let line = formatter.format(&entry);
        match entry.level {
            LogLevel::Error => {
                let stderr = std::io::stderr();
                let mut handle = stderr.lock();
                writeln!(handle, "{line}")?;
            }
            _ => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                writeln!(handle, "{line}")?;
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stdout().flush()?;
        std::io::stderr().flush()?;
        Ok(())
    }
}

struct PlainFormatter;

impl LogFormatter for PlainFormatter {
    fn format(&self, entry: &LogEntry) -> String {
        let ts = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        format!(
            "{ts} {:5} [{}] {}",
            entry.level.as_str(),
            entry.target,
            entry.message,
        )
    }
}

pub struct FileOutput {
    file: File,
    rotation: LogRotation,
    current_size: u64,
    file_count: u32,
}

impl FileOutput {
    pub fn new(path: PathBuf, rotation: LogRotation) -> std::io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        let current_size = file.metadata().map(|m| m.len()).unwrap_or(0);
        Ok(Self { file, rotation, current_size, file_count: 0 })
    }

    fn rotate(&mut self) -> std::io::Result<()> {
        match &self.rotation {
            LogRotation::Size { max_bytes, max_files } => {
                if self.current_size >= *max_bytes {
                    let path = self.file.metadata()?.file_type()?;
                    let _ = path;
                    self.current_size = 0;
                    self.file_count = (self.file_count + 1) % max_files.max(&1);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl LogOutput for FileOutput {
    fn write(&mut self, entry: LogEntry) -> std::io::Result<()> {
        let formatter = JsonFormatter;
        let line = formatter.format(&entry);
        writeln!(self.file, "{line}")?;
        self.current_size += line.len() as u64 + 1;
        self.rotate()?;
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

pub struct TcpOutput {
    addr: String,
    stream: Option<TcpStream>,
}

impl TcpOutput {
    pub fn new(addr: String) -> Self {
        let stream = TcpStream::connect(&addr).ok();
        Self { addr, stream }
    }

    fn reconnect(&mut self) {
        if self.stream.is_none() {
            self.stream = TcpStream::connect(&self.addr).ok();
        }
    }
}

impl LogOutput for TcpOutput {
    fn write(&mut self, entry: LogEntry) -> std::io::Result<()> {
        self.reconnect();
        let formatter = JsonFormatter;
        let line = formatter.format(&entry);
        if let Some(ref mut stream) = self.stream {
            writeln!(stream, "{line}")?;
        }
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(ref mut stream) = self.stream {
            stream.flush()?;
        }
        Ok(())
    }
}

pub enum LogRotation {
    None,
    Size { max_bytes: u64, max_files: u32 },
    Daily { max_days: u32 },
    Hourly { max_hours: u32 },
}

pub struct Logger {
    level: LogLevel,
    formatter: Box<dyn LogFormatter>,
    outputs: Vec<Box<dyn LogOutput>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            level: LogLevel::Info,
            formatter: Box::new(DefaultFormatter),
            outputs: vec![Box::new(ConsoleOutput::new(true))],
        }
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    pub fn add_output(&mut self, output: Box<dyn LogOutput>) {
        self.outputs.push(output);
    }

    pub fn set_formatter(&mut self, formatter: Box<dyn LogFormatter>) {
        self.formatter = formatter;
    }

    pub fn log(&mut self, level: LogLevel, target: &str, message: &str) {
        if !level.should_log(&self.level) {
            return;
        }
        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            target: target.to_string(),
            message: message.to_string(),
            module: std::module_path!().to_string(),
            file: file!().to_string(),
            line: line!(),
            fields: HashMap::new(),
        };
        self.outputs.retain_mut(|output| {
            output.write(entry.clone()).is_ok()
        });
    }

    pub fn error(&mut self, msg: &str) {
        self.log(LogLevel::Error, "klyron", msg);
    }

    pub fn warn(&mut self, msg: &str) {
        self.log(LogLevel::Warn, "klyron", msg);
    }

    pub fn info(&mut self, msg: &str) {
        self.log(LogLevel::Info, "klyron", msg);
    }

    pub fn debug(&mut self, msg: &str) {
        self.log(LogLevel::Debug, "klyron", msg);
    }

    pub fn trace(&mut self, msg: &str) {
        self.log(LogLevel::Trace, "klyron", msg);
    }

    pub fn flush(&mut self) {
        for output in &mut self.outputs {
            let _ = output.flush();
        }
    }

    pub fn global() -> &'static Mutex<Logger> {
        &GLOBAL_LOGGER
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

pub fn init_logger(level: LogLevel) {
    let mut logger = GLOBAL_LOGGER.lock().unwrap_or_else(|e| e.into_inner());
    logger.set_level(level);
}

pub fn set_log_level(level: LogLevel) {
    let mut logger = GLOBAL_LOGGER.lock().unwrap_or_else(|e| e.into_inner());
    logger.set_level(level);
}
