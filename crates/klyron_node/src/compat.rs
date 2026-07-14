use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct NodeCompat {
    pub argv: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: PathBuf,
    pub exec_path: PathBuf,
    pub version: String,
}

impl Default for NodeCompat {
    fn default() -> Self { Self::new() }
}

impl NodeCompat {
    pub fn new() -> Self {
        Self {
            argv: std::env::args().collect(),
            env: std::env::vars().collect(),
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            exec_path: std::env::current_exe().unwrap_or_else(|_| PathBuf::from("klyron")),
            version: "22.0.0".to_string(),
        }
    }

    pub fn process_argv(&self) -> &[String] { &self.argv }
    pub fn process_env(&self) -> &HashMap<String, String> { &self.env }
    pub fn process_cwd(&self) -> &Path { &self.cwd }
    pub fn process_version(&self) -> &str { &self.version }
    pub fn process_pid(&self) -> u32 { std::process::id() }
    pub fn process_ppid(&self) -> u32 { 0 }
    pub fn process_platform(&self) -> &'static str {
        if cfg!(target_os = "linux") { "linux" }
        else if cfg!(target_os = "macos") { "darwin" }
        else if cfg!(target_os = "windows") { "win32" }
        else { "unknown" }
    }
    pub fn process_arch(&self) -> &'static str {
        if cfg!(target_arch = "x86_64") { "x64" }
        else if cfg!(target_arch = "aarch64") { "arm64" }
        else { "unknown" }
    }
    pub fn process_exit(&self, code: i32) -> ! { std::process::exit(code) }

    pub fn process_uptime(&self) -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }

    pub fn process_hrtime(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    pub fn process_memory_usage(&self) -> HashMap<String, u64> {
        let mut usage = HashMap::new();
        usage.insert("rss".to_string(), 0);
        usage.insert("heapTotal".to_string(), 0);
        usage.insert("heapUsed".to_string(), 0);
        usage.insert("external".to_string(), 0);
        usage
    }

    pub fn get_env(&self, key: &str) -> Option<String> {
        self.env.get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.insert(key.to_string(), val.to_string());
    }

    pub fn unset_env(&mut self, key: &str) {
        self.env.remove(key);
    }

    pub fn cwd_str(&self) -> String {
        self.cwd.to_string_lossy().to_string()
    }

    pub fn chdir(&mut self, dir: &Path) -> anyhow::Result<()> {
        std::env::set_current_dir(dir)?;
        self.cwd = dir.to_path_buf();
        Ok(())
    }

    pub fn next_tick<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(f);
    }
}

pub fn create_buffer(size: usize) -> Vec<u8> {
    vec![0u8; size]
}

pub fn buffer_from(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

pub fn buffer_from_string(s: &str, encoding: &str) -> Vec<u8> {
    match encoding {
        "hex" => hex::decode(s).unwrap_or_default(),
        "base64" => base64_decode(s),
        _ => s.as_bytes().to_vec(),
    }
}

pub fn buffer_to_string(data: &[u8], encoding: &str) -> String {
    match encoding {
        "hex" => hex::encode(data),
        "base64" => base64_encode(data),
        _ => String::from_utf8_lossy(data).to_string(),
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn base64_decode(s: &str) -> Vec<u8> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s).unwrap_or_default()
}
