use std::path::Path;
use std::time::SystemTime;

pub struct StaticFile {
    pub path: String,
    pub content: Vec<u8>,
    pub content_type: String,
    pub modified: Option<SystemTime>,
    pub size: u64,
}

impl StaticFile {
    pub fn load(path: &str, base_dir: &Path) -> anyhow::Result<Option<Self>> {
        let full_path = base_dir.join(path.trim_start_matches('/'));
        if !full_path.exists() || !full_path.is_file() {
            return Ok(None);
        }
        let metadata = full_path.metadata()?;
        let modified = metadata.modified().ok();
        let size = metadata.len();
        let content = std::fs::read(&full_path)?;
        let ext = full_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let content_type = mime_type(ext).to_string();

        Ok(Some(Self {
            path: path.to_string(),
            content,
            content_type,
            modified,
            size,
        }))
    }

    pub fn etag(&self) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.content.hash(&mut hasher);
        format!("\"{:x}\"", hasher.finish())
    }
}

fn mime_type(ext: &str) -> &'static str {
    match ext {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" | "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

pub struct StaticFileServer {
    root: std::path::PathBuf,
}

impl StaticFileServer {
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            root: root.into(),
        }
    }

    pub fn serve(&self, request_path: &str) -> anyhow::Result<Option<StaticFile>> {
        let clean_path = request_path
            .trim_start_matches('/')
            .split('?')
            .next()
            .unwrap_or("");

        if clean_path.is_empty() {
            return StaticFile::load("index.html", &self.root);
        }

        if clean_path.contains("..") {
            return Ok(None);
        }

        let result = StaticFile::load(clean_path, &self.root)?;
        if result.is_some() {
            return Ok(result);
        }

        StaticFile::load("index.html", &self.root)
    }
}
