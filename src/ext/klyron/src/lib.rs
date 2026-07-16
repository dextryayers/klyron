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

    fn version_impl() -> String { env!("CARGO_PKG_VERSION").to_string() }
    fn arch_impl() -> String { std::env::consts::ARCH.to_string() }
    fn platform_impl() -> String { std::env::consts::OS.to_string() }

    #[test]
    fn test_klyron_version_not_empty() {
        let v = version_impl();
        assert!(!v.is_empty());
        assert!(v.contains('.'));
    }

    #[test]
    fn test_klyron_arch_not_empty() {
        assert!(!arch_impl().is_empty());
    }

    #[test]
    fn test_klyron_platform_not_empty() {
        assert!(!platform_impl().is_empty());
    }

    #[test]
    fn test_klyron_arch_known() {
        let a = arch_impl();
        assert!(a == "x86_64" || a == "aarch64" || a == "x86");
    }

    #[test]
    fn test_klyron_platform_known() {
        let p = platform_impl();
        assert!(p == "linux" || p == "macos" || p == "windows");
    }

    #[test]
    fn test_klyron_version_semver() {
        let v = version_impl();
        let parts: Vec<&str> = v.split('.').collect();
        assert!(parts.len() >= 2);
    }
}
