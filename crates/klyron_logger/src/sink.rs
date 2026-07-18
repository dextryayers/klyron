use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

use flate2::write::GzEncoder;
use flate2::Compression;

pub trait LogSink: Send + 'static {
    fn write(&mut self, line: &str) -> anyhow::Result<()>;
    fn flush(&mut self) -> anyhow::Result<()>;
}

pub struct ConsoleSink {
    write_stderr: bool,
}

impl ConsoleSink {
    pub fn new() -> Self {
        ConsoleSink { write_stderr: false }
    }

    pub fn with_stderr() -> Self {
        ConsoleSink { write_stderr: true }
    }
}

impl Default for ConsoleSink {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSink for ConsoleSink {
    fn write(&mut self, line: &str) -> anyhow::Result<()> {
        if self.write_stderr {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct FileSink {
    file: std::fs::File,
    path: String,
    size: u64,
    max_size: u64,
    backup_count: u32,
    compression: bool,
}

impl FileSink {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Self::open_with_config(
            path,
            10 * 1024 * 1024,
            5,
            true,
        )
    }

    pub fn open_with_config(
        path: &str,
        max_size: u64,
        backup_count: u32,
        compression: bool,
    ) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let size = file.metadata()?.len();
        Ok(FileSink {
            file,
            path: path.to_string(),
            size,
            max_size,
            backup_count,
            compression,
        })
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

impl LogSink for FileSink {
    fn write(&mut self, line: &str) -> anyhow::Result<()> {
        if self.size >= self.max_size {
            self.rotate()?;
        }
        let bytes = line.as_bytes();
        self.file.write_all(bytes)?;
        self.file.write_all(b"\n")?;
        self.size += bytes.len() as u64 + 1;
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.file.flush()?;
        Ok(())
    }
}

#[cfg(unix)]
pub struct SyslogSink {
    facility: i32,
    options: i32,
}

#[cfg(unix)]
impl SyslogSink {
    pub fn new(identity: &str) -> Self {
        let syslog_identity = std::ffi::CString::new(identity).unwrap();
        unsafe {
            libc::openlog(syslog_identity.as_ptr(), libc::LOG_PID, libc::LOG_USER);
        }
        SyslogSink {
            facility: libc::LOG_USER,
            options: libc::LOG_PID,
        }
    }

    fn priority_for_line(_line: &str) -> i32 {
        libc::LOG_INFO
    }
}

#[cfg(unix)]
impl LogSink for SyslogSink {
    fn write(&mut self, line: &str) -> anyhow::Result<()> {
        let c_msg = std::ffi::CString::new(line).unwrap_or_default();
        unsafe {
            libc::syslog(Self::priority_for_line(line), c_msg.as_ptr());
        }
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(unix)]
impl Drop for SyslogSink {
    fn drop(&mut self) {
        unsafe {
            libc::closelog();
        }
    }
}

pub struct MultiSink {
    sinks: Vec<Arc<std::sync::Mutex<Box<dyn LogSink>>>>,
}

impl MultiSink {
    pub fn new() -> Self {
        MultiSink { sinks: Vec::new() }
    }

    pub fn add<S: LogSink>(&mut self, sink: S) {
        self.sinks
            .push(Arc::new(std::sync::Mutex::new(Box::new(sink))));
    }

    pub fn add_arc(&mut self, sink: Arc<std::sync::Mutex<Box<dyn LogSink>>>) {
        self.sinks.push(sink);
    }
}

impl Default for MultiSink {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSink for MultiSink {
    fn write(&mut self, line: &str) -> anyhow::Result<()> {
        for sink in &self.sinks {
            if let Ok(mut s) = sink.lock() {
                let _ = s.write(line);
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        for sink in &self.sinks {
            if let Ok(mut s) = sink.lock() {
                let _ = s.flush();
            }
        }
        Ok(())
    }
}

pub enum LogMsg {
    Line(String),
    Flush(tokio::sync::oneshot::Sender<()>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_sink() {
        let mut sink = ConsoleSink::new();
        assert!(sink.write("test message").is_ok());
    }

    #[test]
    fn test_console_sink_stderr() {
        let mut sink = ConsoleSink::with_stderr();
        assert!(sink.write("test stderr").is_ok());
    }

    #[test]
    fn test_file_sink_create() {
        let dir = std::env::temp_dir();
        let path = dir.join("_klyron_test_file_sink.log");
        let _ = std::fs::remove_file(&path);
        let mut sink = FileSink::open(path.to_str().unwrap()).unwrap();
        assert!(sink.write("test line").is_ok());
        assert!(sink.flush().is_ok());
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_multi_sink() {
        let mut multi = MultiSink::new();
        multi.add(ConsoleSink::new());
        assert!(multi.write("multi test").is_ok());
    }

    #[test]
    fn test_file_sink_rotation() {
        let dir = std::env::temp_dir();
        let path = dir.join("_klyron_test_rotation.log");
        let _ = std::fs::remove_file(&path);
        {
            let mut sink = FileSink::open_with_config(path.to_str().unwrap(), 10, 3, false).unwrap();
            for i in 0..20 {
                assert!(sink.write(&format!("line {i}")).is_ok());
            }
        }
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }
}
