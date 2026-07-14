#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel { Trace = 0, Debug = 1, Info = 2, Warn = 3, Error = 4, Fatal = 5 }
impl LogLevel {
    pub fn as_str(&self) -> &'static str { match self { LogLevel::Trace => "TRACE", LogLevel::Debug => "DEBUG", LogLevel::Info => "INFO", LogLevel::Warn => "WARN", LogLevel::Error => "ERROR", LogLevel::Fatal => "FATAL" } }
}

#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub min_level: LogLevel,
    pub json_output: bool,
    pub file_path: Option<String>,
    pub include_location: bool,
    pub color_enabled: bool,
}
impl Default for LoggerConfig {
    fn default() -> Self { Self { min_level: LogLevel::Info, json_output: false, file_path: None, include_location: false, color_enabled: true } }
}

pub struct Logger;
impl Logger {
    pub fn new() -> Self { Self }
    pub fn with_config(_config: LoggerConfig) -> anyhow::Result<Self> { Ok(Self) }
    pub fn with_min_level(self, _level: LogLevel) -> Self { self }
    pub fn with_json(self, _enabled: bool) -> Self { self }
    pub fn with_file(self, _path: &std::path::Path) -> anyhow::Result<Self> { Ok(self) }
    pub fn trace(&self, _msg: &str) {}
    pub fn debug(&self, _msg: &str) {}
    pub fn info(&self, _msg: &str) {}
    pub fn warn(&self, _msg: &str) {}
    pub fn error(&self, _msg: &str) {}
    pub fn fatal(&self, _msg: &str) {}
}
impl Default for Logger { fn default() -> Self { Self::new() } }
