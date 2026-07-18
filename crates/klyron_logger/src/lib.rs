pub mod format;
pub mod sink;

pub use format::*;
pub use sink::*;

use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use crossbeam_channel::Sender;

#[derive(Debug, Clone)]
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
    fn default() -> Self {
        LoggerConfig {
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

pub struct Logger {
    min_level: LogLevel,
    json_output: bool,
    include_location: bool,
    color_enabled: bool,
    async_tx: Sender<LogMsg>,
    fields: Option<serde_json::Value>,
    file_sink: Option<Arc<std::sync::Mutex<FileSink>>>,
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
                    LogMsg::Flush(tx) => {
                        let _ = tx.send(());
                    }
                }
            }
        });

        Logger {
            min_level: LogLevel::Info,
            json_output: false,
            include_location: false,
            color_enabled: true,
            async_tx,
            fields: None,
            file_sink: None,
        }
    }

    pub fn with_config(config: LoggerConfig) -> anyhow::Result<Self> {
        let file_sink = if let Some(ref path) = config.file_path {
            let sink = FileSink::open_with_config(
                path,
                config.max_file_size,
                config.max_backup_files,
                config.compression_enabled,
            )?;
            Some(Arc::new(std::sync::Mutex::new(sink)))
        } else {
            None
        };

        let (async_tx, async_rx) = crossbeam_channel::unbounded::<LogMsg>();
        let logger_file = file_sink.clone();

        std::thread::spawn(move || {
            for msg in async_rx {
                match msg {
                    LogMsg::Line(line) => {
                        if let Some(ref sink) = logger_file {
                            if let Ok(mut f) = sink.lock() {
                                let _ = f.write(&line);
                            }
                        }
                    }
                    LogMsg::Flush(tx) => {
                        let _ = tx.send(());
                    }
                }
            }
        });

        Ok(Logger {
            min_level: config.min_level,
            json_output: config.json_output,
            include_location: config.include_location,
            color_enabled: config.color_enabled,
            async_tx,
            fields: None,
            file_sink,
        })
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
        let sink = FileSink::open(path.to_str().unwrap())?;
        let sink = Arc::new(std::sync::Mutex::new(sink));

        let (async_tx, async_rx) = crossbeam_channel::unbounded::<LogMsg>();
        let file_sink = sink.clone();

        std::thread::spawn(move || {
            for msg in async_rx {
                match msg {
                    LogMsg::Line(line) => {
                        if let Ok(mut f) = file_sink.lock() {
                            let _ = f.write(&line);
                        }
                    }
                    LogMsg::Flush(tx) => {
                        let _ = tx.send(());
                    }
                }
            }
        });

        self.async_tx = async_tx;
        self.file_sink = Some(sink);
        Ok(self)
    }

    pub fn with_location(mut self, enabled: bool) -> Self {
        self.include_location = enabled;
        self
    }

    pub fn with_fields(mut self, fields: serde_json::Value) -> Self {
        self.fields = Some(fields);
        self
    }

    pub fn is_enabled(&self, level: LogLevel) -> bool {
        level >= self.min_level
    }

    pub fn trace(&self, msg: &str) {
        self.log(LogLevel::Trace, msg, None);
    }

    pub fn debug(&self, msg: &str) {
        self.log(LogLevel::Debug, msg, None);
    }

    pub fn info(&self, msg: &str) {
        self.log(LogLevel::Info, msg, None);
    }

    pub fn warn(&self, msg: &str) {
        self.log(LogLevel::Warn, msg, None);
    }

    pub fn error(&self, msg: &str) {
        self.log(LogLevel::Error, msg, None);
    }

    pub fn fatal(&self, msg: &str) {
        self.log(LogLevel::Fatal, msg, None);
    }

    pub fn trace_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Trace, msg, Some(meta));
    }

    pub fn debug_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Debug, msg, Some(meta));
    }

    pub fn info_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Info, msg, Some(meta));
    }

    pub fn warn_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Warn, msg, Some(meta));
    }

    pub fn error_with(&self, msg: &str, meta: serde_json::Value) {
        self.log(LogLevel::Error, msg, Some(meta));
    }

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

        let mut entry = LogEntry::new(level, msg);
        if let Some(f) = fields {
            entry = entry.with_fields(f);
        }

        let output = if self.json_output {
            entry.to_json()
        } else {
            entry.to_colored_text(self.color_enabled)
        };

        match level {
            LogLevel::Error | LogLevel::Fatal => eprintln!("{output}"),
            _ => println!("{output}"),
        }

        let _ = self.async_tx.send(LogMsg::Line(output));
    }

    pub fn flush(&self) {
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        let _ = self.async_tx.send(LogMsg::Flush(tx));
        let _ = rx.try_recv();
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
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
            let logger = Logger::new().with_file(&path).unwrap();
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

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("WARNING"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("unknown"), None);
    }

    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new(LogLevel::Info, "hello");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "hello");
    }

    #[test]
    fn test_log_entry_with_fields() {
        let entry = LogEntry::new(LogLevel::Debug, "test")
            .with_fields(serde_json::json!({"key": "value"}));
        assert!(entry.fields.is_some());
    }

    #[test]
    fn test_log_entry_to_json() {
        let entry = LogEntry::new(LogLevel::Info, "json check");
        let json = entry.to_json();
        assert!(json.contains("INFO"));
        assert!(json.contains("json check"));
    }

    #[test]
    fn test_log_entry_to_text() {
        let entry = LogEntry::new(LogLevel::Info, "text check");
        let text = entry.to_text();
        assert!(text.contains("INFO"));
        assert!(text.contains("text check"));
    }
}
