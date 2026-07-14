use crate::types::{InstallOptions, PackageManager};

#[derive(Debug, Clone)]
pub struct PmBuilder {
    manager: Option<PackageManager>,
    path: std::path::PathBuf,
    options: InstallOptions,
}

impl PmBuilder {
    pub fn new(path: &std::path::Path) -> Self {
        Self { manager: None, path: path.to_path_buf(), options: InstallOptions { dev: false, global: false, frozen_lockfile: false } }
    }

    pub fn manager(mut self, m: PackageManager) -> Self { self.manager = Some(m); self }
    pub fn dev(mut self, v: bool) -> Self { self.options.dev = v; self }
    pub fn frozen_lockfile(mut self, v: bool) -> Self { self.options.frozen_lockfile = v; self }
    pub fn build(self) -> super::PackageManagerClient {
        let pm = self.manager.unwrap_or_else(|| super::detect(&self.path));
        super::PackageManagerClient::new(pm, self.path)
    }
}
