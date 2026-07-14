use crate::types::PackageManager;

#[derive(Debug, Clone)]
pub struct PmConfig {
    pub default_manager: PackageManager,
    pub lockfile_check: bool,
    pub auto_install: bool,
}

impl Default for PmConfig {
    fn default() -> Self {
        Self { default_manager: PackageManager::Npm, lockfile_check: true, auto_install: false }
    }
}
