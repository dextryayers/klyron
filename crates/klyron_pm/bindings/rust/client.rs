use std::path::Path;
use crate::{PackageManager, detect, install_cmd, add_cmd};

pub struct PackageManagerClient {
    manager: PackageManager,
    path: std::path::PathBuf,
}

impl PackageManagerClient {
    pub fn new(manager: PackageManager, path: std::path::PathBuf) -> Self {
        Self { manager, path }
    }

    pub fn manager(&self) -> PackageManager { self.manager }

    pub fn install(&self) -> String { install_cmd(self.manager).to_string() }

    pub fn add(&self, dev: bool) -> String { add_cmd(self.manager, dev).to_string() }

    pub fn path(&self) -> &Path { &self.path }

    pub fn detect_from_path() -> PackageManager {
        detect(&std::env::current_dir().unwrap_or_default())
    }
}
