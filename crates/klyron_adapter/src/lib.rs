use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub mod adapters;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub release: bool,
    pub minify: bool,
    pub sourcemap: bool,
    pub target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScaffoldOptions {
    pub dir: PathBuf,
    pub version: Option<String>,
    pub template_vars: HashMap<String, String>,
}

impl Default for ScaffoldOptions {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("."),
            version: None,
            template_vars: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkInfo {
    pub name: String,
    pub version: String,
    pub detection_file: String,
    pub supported_versions: Vec<String>,
    pub default_version: String,
    pub kind: FrameworkKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FrameworkKind {
    Frontend,
    Backend,
    Fullstack,
    Polyglot,
}

#[async_trait]
pub trait FrameworkAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    fn detect(&self, dir: &Path) -> bool;
    fn supported_versions(&self) -> Vec<&'static str>;
    fn default_version(&self) -> &'static str;
    fn kind(&self) -> FrameworkKind;

    async fn dev(&self, dir: &Path, port: Option<u16>) -> anyhow::Result<()>;
    async fn build(&self, dir: &Path, opts: BuildOptions) -> anyhow::Result<()>;
    async fn test(&self, dir: &Path, filter: Option<&str>) -> anyhow::Result<()>;
    async fn lint(&self, dir: &Path, fix: bool) -> anyhow::Result<()>;
    async fn format(&self, dir: &Path, write: bool) -> anyhow::Result<()>;
    async fn scaffold(&self, name: &str, options: ScaffoldOptions) -> anyhow::Result<()>;
}

pub struct AdapterRegistry {
    adapters: HashMap<&'static str, Arc<dyn FrameworkAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn register(&mut self, adapter: Arc<dyn FrameworkAdapter>) {
        self.adapters.insert(adapter.name(), adapter);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn FrameworkAdapter>> {
        self.adapters.get(name).cloned()
    }

    pub fn detect(&self, dir: &Path) -> Vec<Arc<dyn FrameworkAdapter>> {
        self.adapters
            .values()
            .filter(|a| a.detect(dir))
            .cloned()
            .collect()
    }

    pub fn all(&self) -> Vec<Arc<dyn FrameworkAdapter>> {
        self.adapters.values().cloned().collect()
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.adapters.keys().copied().collect()
    }

    pub fn by_kind(&self, kind: FrameworkKind) -> Vec<Arc<dyn FrameworkAdapter>> {
        self.adapters
            .values()
            .filter(|a| a.kind() == kind)
            .cloned()
            .collect()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run_dev(adapter: Arc<dyn FrameworkAdapter>, dir: &Path, port: Option<u16>) -> anyhow::Result<()> {
    adapter.dev(dir, port).await
}

pub async fn run_build(adapter: Arc<dyn FrameworkAdapter>, dir: &Path, opts: BuildOptions) -> anyhow::Result<()> {
    adapter.build(dir, opts).await
}

pub async fn run_test(adapter: Arc<dyn FrameworkAdapter>, dir: &Path, filter: Option<&str>) -> anyhow::Result<()> {
    adapter.test(dir, filter).await
}

pub async fn run_lint(adapter: Arc<dyn FrameworkAdapter>, dir: &Path, fix: bool) -> anyhow::Result<()> {
    adapter.lint(dir, fix).await
}

pub async fn run_format(adapter: Arc<dyn FrameworkAdapter>, dir: &Path, write: bool) -> anyhow::Result<()> {
    adapter.format(dir, write).await
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAdapter;

    #[async_trait]
    impl FrameworkAdapter for MockAdapter {
        fn name(&self) -> &'static str { "mock" }
        fn detect(&self, _dir: &Path) -> bool { true }
        fn supported_versions(&self) -> Vec<&'static str> { vec!["1.0"] }
        fn default_version(&self) -> &'static str { "1.0" }
        fn kind(&self) -> FrameworkKind { FrameworkKind::Frontend }
        async fn dev(&self, _dir: &Path, _port: Option<u16>) -> anyhow::Result<()> { Ok(()) }
        async fn build(&self, _dir: &Path, _opts: BuildOptions) -> anyhow::Result<()> { Ok(()) }
        async fn test(&self, _dir: &Path, _filter: Option<&str>) -> anyhow::Result<()> { Ok(()) }
        async fn lint(&self, _dir: &Path, _fix: bool) -> anyhow::Result<()> { Ok(()) }
        async fn format(&self, _dir: &Path, _write: bool) -> anyhow::Result<()> { Ok(()) }
        async fn scaffold(&self, _name: &str, _opts: ScaffoldOptions) -> anyhow::Result<()> { Ok(()) }
    }

    #[test]
    fn test_registry() {
        let mut reg = AdapterRegistry::new();
        reg.register(Arc::new(MockAdapter));
        assert!(reg.get("mock").is_some());
        assert_eq!(reg.names(), vec!["mock"]);
        assert_eq!(reg.by_kind(FrameworkKind::Frontend).len(), 1);
        assert_eq!(reg.by_kind(FrameworkKind::Backend).len(), 0);
    }

    #[test]
    fn test_detect() {
        let mut reg = AdapterRegistry::new();
        reg.register(Arc::new(MockAdapter));
        let detected = reg.detect(Path::new("/tmp"));
        assert_eq!(detected.len(), 1);
    }

    #[tokio::test]
    async fn test_run_helpers() {
        let adapter = Arc::new(MockAdapter);
        assert!(run_dev(adapter.clone(), Path::new("."), None).await.is_ok());
        assert!(run_build(adapter.clone(), Path::new("."), BuildOptions {
            release: false, minify: false, sourcemap: false, target: None
        }).await.is_ok());
        assert!(run_test(adapter.clone(), Path::new("."), None).await.is_ok());
        assert!(run_lint(adapter.clone(), Path::new("."), false).await.is_ok());
        assert!(run_format(adapter.clone(), Path::new("."), false).await.is_ok());
    }

    #[test]
    fn test_framework_info() {
        let info = FrameworkInfo {
            name: "react".into(),
            version: "19.0.0".into(),
            detection_file: "vite.config.*".into(),
            supported_versions: vec!["18.0".into(), "19.0".into()],
            default_version: "19.0".into(),
            kind: FrameworkKind::Frontend,
        };
        assert_eq!(info.name, "react");
        assert_eq!(info.kind, FrameworkKind::Frontend);
    }
}
