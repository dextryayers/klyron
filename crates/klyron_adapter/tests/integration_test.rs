use klyron_adapter::{AdapterRegistry, AdapterInfo, FrameworkAdapter};

#[test]
fn test_adapter_registry_new() {
    // Verifying that an adapter registry is created without panicking
    let registry = AdapterRegistry::new();
    assert!(registry.frameworks().is_empty(), "fresh registry should have no frameworks");
}

#[test]
fn test_adapter_info_default() {
    let info = AdapterInfo {
        name: "test".into(),
        version: "0.1.0".into(),
        description: "Test adapter".into(),
    };
    assert_eq!(info.name, "test");
    assert_eq!(info.version, "0.1.0");
}

#[test]
fn test_adapter_detect_no_project() {
    let registry = AdapterRegistry::new();
    let dir = std::env::temp_dir().join("klyron_adapter_test_nonexistent");
    let result = registry.detect(&dir);
    assert!(result.is_none(), "detect on nonexistent dir should be None");
}

#[test]
fn test_adapter_frameworks_list() {
    // Use the Klyron-built adapter list from the crate
    let builtins = klyron_adapter::builtin_adapters();
    assert!(!builtins.is_empty(), "there should be at least some built-in adapters");
    assert!(builtins.iter().any(|a| a.name.contains("express") || a.name.contains("react")), "expected express or react adapter");
}
