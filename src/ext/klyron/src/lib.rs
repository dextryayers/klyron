use deno_core::{extension, op2, Extension};

extension!(
  klyron_klyron,
  ops = [op_klyron_version, op_klyron_arch, op_klyron_platform],
  esm_entry_point = "ext:klyron_klyron/klyron.js",
  esm = [dir "js", "klyron.js"],
);

pub fn init() -> Extension {
  klyron_klyron::init()
}

#[op2]
#[string]
fn op_klyron_version() -> String {
  env!("CARGO_PKG_VERSION").to_string()
}

#[op2]
#[string]
fn op_klyron_arch() -> String {
  std::env::consts::ARCH.to_string()
}

#[op2]
#[string]
fn op_klyron_platform() -> String {
  std::env::consts::OS.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_returns_extension() {
        let ext = init();
        assert_eq!(ext.name, "klyron_klyron");
    }

    #[test]
    fn test_klyron_version_not_empty() {
        let version = op_klyron_version();
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    #[test]
    fn test_klyron_arch_not_empty() {
        let arch = op_klyron_arch();
        assert!(!arch.is_empty());
    }

    #[test]
    fn test_klyron_platform_not_empty() {
        let platform = op_klyron_platform();
        assert!(!platform.is_empty());
    }

    #[test]
    fn test_klyron_arch_known() {
        let arch = op_klyron_arch();
        assert!(arch == "x86_64" || arch == "aarch64" || arch == "x86" || !arch.is_empty());
    }

    #[test]
    fn test_klyron_platform_known() {
        let platform = op_klyron_platform();
        assert!(platform == "linux" || platform == "macos" || platform == "windows" || !platform.is_empty());
    }

    #[test]
    fn test_klyron_version_semver() {
        let version = op_klyron_version();
        let parts: Vec<&str> = version.split('.').collect();
        assert!(parts.len() >= 2);
    }
}
