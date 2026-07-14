use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use flate2::write::GzEncoder;
use flate2::Compression;
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
    #[inline]
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoggerConfig {
    pub min_level: LogLevel,
    pub json_output: bool,
    pub file_path: Option<String>,
    pub include_location: bool,
    pub max_file_size: u64,
    pub color_enabled: bool,
    pub compression_enabled: bool,
    pub max_backup_files: u32,
}

impl Default for LoggerConfig {
    #[inline]
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            json_output: false,
            file_path: None,
            include_location: false,
            max_file_size: 10 * 1024 * 1024,
            color_enabled: true,
            compression_enabled: true,
            max_backup_files: 5,
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
    fields: Option<serde_json::Value>,
}

struct LogFile {
    file: std::fs::File,
    path: String,
    size: u64,
    max_size: u64,
    backup_count: u32,
    compression: bool,
}

impl LogFile {
    fn open(path: &str, max_size: u64, backup_count: u32, compression: bool) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let size = file.metadata()?.len();
        Ok(Self { file, path: path.to_string(), size, max_size, backup_count, compression })
    }

    fn write_line(&mut self, line: &str) -> anyhow::Result<()> {
        if self.size >= self.max_size {
            self.rotate()?;
        }
        let bytes = line.as_bytes();
        self.file.write_all(bytes)?;
        self.file.write_all(b"\n")?;
        self.size += bytes.len() as u64 + 1;
        Ok(())
    }

    fn rotate(&mut self) -> anyhow::Result<()> {
        use std::fs;

        for i in (1..self.backup_count).rev() {
            let src = format!("{}.{}", self.path, i);
            let dst = format!("{}.{}", self.path, i + 1);
            if fs::metadata(&src).is_ok() {
                let _ = fs::rename(&src, &dst);
            }
        }

        let first_backup = format!("{}.1", self.path);
        let _ = fs::rename(&self.path, &first_backup);

        if self.compression {
            let dst_path = format!("{}.1.gz", self.path);
            if let Ok(mut src) = fs::File::open(&first_backup) {
                let mut buf = Vec::new();
                if src.read_to_end(&mut buf).is_ok() {
                    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                    if encoder.write_all(&buf).is_ok() {
                        if let Ok(compressed) = encoder.finish() {
                            let _ = fs::write(&dst_path, &compressed);
                        }
                    }
                }
            }
            let _ = fs::remove_file(&first_backup);
        }

        self.file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;
        self.size = 0;
        Ok(())
    }
}

enum LogMsg {
    Line(String),
    Flush(tokio::sync::oneshot::Sender<()>),
}

pub struct Logger {
    min_level: LogLevel,
    json_output: bool,
    include_location: bool,
    color_enabled: bool,
    async_tx: crossbeam_channel::Sender<LogMsg>,
    fields: Option<serde_json::Value>,
    otel_endpoint: Option<String>,
}

impl Logger {
    pub fn new() -> Self {
        let (async_tx, async_rx) = crossbeam_channel::unbounded::<LogMsg>();

        std::thread::spawn(move || {
            for msg in async_rx {
                match msg {
                    LogMsg::Line(line) => {
                        println!("{line}");
                    }
                    LogMsg::Flush(tx) => { let _ = tx.send(()); }
                }
            }
        });

        Self {
            min_level: LogLevel::Info,
            json_output: false,
            include_location: false,
            color_enabled: true,
            async_tx,
            fields: None,
            otel_endpoint: None,
        }
    }

    pub fn with_config(config: LoggerConfig) -> anyhow::Result<Self> {
        let log_file = if let Some(ref path) = config.file_path {
            let lf = LogFile::open(
                path,
                config.max_file_size,
                config.max_backup_files,
                config.compression_enabled,
            )?;
            Some(Arc::new(std::sync::Mutex::new(lf)))
        } else {
            None
        };

        let (async_tx, async_rx) = crossbeam_channel::unbounded::<LogMsg>();
        let logger_file = log_file.clone();

        std::thread::spawn(move || {
            for msg in async_rx {
                match msg {
                    LogMsg::Line(line) => {
                        if let Some(ref lf) = logger_file {
                            if let Ok(mut f) = lf.lock() {
                                let _ = f.write_line(&line);
                            }
                        }
                    }
                    LogMsg::Flush(tx) => { let _ = tx.send(()); }
                }
            }
        });

        Ok(Self {
            min_level: config.min_level,
            json_output: config.json_output,
            include_location: config.include_location,
            color_enabled: config.color_enabled,
            async_tx,
            fields: None,
            otel_endpoint: None,
        })
    }

    #[inline]
    pub fn with_min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    #[inline]
    pub fn with_json(mut self, enabled: bool) -> Self {
        self.json_output = enabled;
        self
    }

    pub fn with_file(mut self, path: &Path) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let lf = LogFile {
            file,
            path: path.to_string_lossy().to_string(),
            size: path.metadata().ok().map(|m| m.len()).unwrap_or(0),
            max_size: 10 * 1024 * 1024,
            backup_count: 5,
            compression: true,
        };
        let lf = std::sync::Mutex::new(lf);
        self.async_tx = {
            let (tx, rx) = crossbeam_channel::unbounded::<LogMsg>();
            std::thread::spawn(move || {
                for msg in rx {
                    match msg {
                        LogMsg::Line(line) => {
                            if let Ok(mut f) = lf.lock() {
                                let _ = f.write_line(&line);
                            }
                        }
                        LogMsg::Flush(tx) => { let _ = tx.send(()); }
                    }
                }
            });
            tx
        };
        Ok(self)
    }

    #[inline]
    pub fn with_location(mut self, enabled: bool) -> Self {
        self.include_location = enabled;
        self
    }

    #[inline]
    pub fn with_fields(mut self, fields: serde_json::Value) -> Self {
        self.fields = Some(fields);
        self
    }

    pub fn with_otel(mut self, endpoint: &str) -> Self {
        self.otel_endpoint = Some(endpoint.to_string());
        self
    }

    #[inline]
    pub fn is_enabled(&self, level: LogLevel) -> bool {
        level >= self.min_level
    }

    #[inline]
    pub fn trace(&self, msg: &str) {
        self.log(LogLevel::Trace, msg, None);
    }

    #[inline]
    pub fn debug(&self, msg: &str) {
        self.log(LogLevel::Debug, msg, None);
    }

    #[inline]
    pub fn info(&self, msg: &str) {
        self.log(LogLevel::Info, msg, None);
    }

    #[inline]
    pub fn warn(&self, msg: &str) {
        self.log(LogLevel::Warn, msg, None);
    }

    #[inline]
    pub fn error(&self, msg: &str) {
        self.log(LogLevel::Error, msg, None);
    }

    #[inline]
    pub fn fatal(&self, msg: &str) {
        self.log(LogLevel::Fatal, msg, None);
    }

    #[inline]
    pub fn trace_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Trace, msg, Some(meta));
    }

    #[inline]
    pub fn debug_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Debug, msg, Some(meta));
    }

    #[inline]
    pub fn info_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Info, msg, Some(meta));
    }

    #[inline]
    pub fn warn_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Warn, msg, Some(meta));
    }

    #[inline]
    pub fn error_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Error, msg, Some(meta));
    }

    #[inline]
    pub fn fatal_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Fatal, msg, Some(meta));
    }

    fn log(&self, level: LogLevel, msg: &str, metadata: Option<serde_json::Value>) {
        if level < self.min_level {
            return;
        }

        let fields = match (&self.fields, metadata) {
            (Some(base), Some(meta)) => {
                let mut merged = base.clone();
                if let Some(obj) = merged.as_object_mut() {
                    if let Some(meta_obj) = meta.as_object() {
                        for (k, v) in meta_obj {
                            obj.insert(k.clone(), v.clone());
                        }
                    }
                }
                Some(merged)
            }
            (Some(base), None) => Some(base.clone()),
            (None, meta) => meta,
        };

        let entry = LogEntry {
            timestamp: Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            level: level.as_str().to_string(),
            message: msg.to_string(),
            module: None,
            file: None,
            line: None,
            fields,
        };

        if self.json_output {
            let json = serde_json::to_string(&entry).unwrap_or_default();
            self.write_output(level, &json);
        } else {
            let ts = &entry.timestamp[..23];
            let formatted = if self.color_enabled {
                let color = match level {
                    LogLevel::Trace => "\x1b[90m",
                    LogLevel::Debug => "\x1b[36m",
                    LogLevel::Info => "\x1b[32m",
                    LogLevel::Warn => "\x1b[33m",
                    LogLevel::Error => "\x1b[31m",
                    LogLevel::Fatal => "\x1b[35m",
                };
                let reset = "\x1b[0m";
                format!("{color}{ts} [{:5}] {}{reset}", entry.level, entry.message)
            } else {
                format!("{ts} [{:5}] {}", entry.level, entry.message)
            };
            self.write_output(level, &formatted);
        }
    }

    fn write_output(&self, level: LogLevel, line: &str) {
        match level {
            LogLevel::Error | LogLevel::Fatal => eprintln!("{line}"),
            _ => println!("{line}"),
        }

        let _ = self.async_tx.send(LogMsg::Line(line.to_string()));
    }

    pub fn flush(&self) {
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        let _ = self.async_tx.send(LogMsg::Flush(tx));
        let _ = rx.try_recv();
    }
}

impl Default for Logger {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub fn info(msg: &str) {
    Logger::new().info(msg);
}

#[inline]
pub fn warn(msg: &str) {
    Logger::new().warn(msg);
}

#[inline]
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

    #[test]
    fn test_log_levels() {
        let logger = Logger::new().with_min_level(LogLevel::Debug);
        logger.debug("debug msg");
        logger.info("info msg");
        logger.warn("warn msg");
        logger.error("error msg");
    }

    #[test]
    fn test_logger_json() {
        let logger = Logger::new().with_json(true);
        logger.info("json test");
    }

    #[test]
    fn test_logger_with_fields() {
        let logger = Logger::new()
            .with_json(true)
            .with_fields(serde_json::json!({"app": "klyron", "version": "1.0"}));
        logger.info("structured log");
    }

    #[test]
    fn test_logger_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("klyron_test.log");
        let _ = std::fs::remove_file(&path);
        {
            let logger = Logger::new()
                .with_file(&path)
                .unwrap();
            logger.info("file test");
            logger.flush();
        }
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_logger_config() {
        let config = LoggerConfig {
            min_level: LogLevel::Debug,
            json_output: true,
            include_location: true,
            compression_enabled: false,
            ..Default::default()
        };
        let logger = Logger::with_config(config).unwrap();
        logger.debug("config test");
    }

    #[test]
    fn test_is_enabled() {
        let logger = Logger::new().with_min_level(LogLevel::Warn);
        assert!(!logger.is_enabled(LogLevel::Trace));
        assert!(!logger.is_enabled(LogLevel::Info));
        assert!(logger.is_enabled(LogLevel::Warn));
        assert!(logger.is_enabled(LogLevel::Error));
        assert!(logger.is_enabled(LogLevel::Fatal));
    }

    #[test]
    fn test_fatal_level() {
        let logger = Logger::new();
        logger.fatal("fatal error");
    }

    #[test]
    fn test_logger_trace() {
        let logger = Logger::new().with_min_level(LogLevel::Trace);
        logger.trace("trace message");
        assert!(logger.is_enabled(LogLevel::Trace));
    }
}
