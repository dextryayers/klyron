use klyron_adapter::{AdapterRegistry, FrameworkKind};

#[test]
fn test_adapter_registry_new() {
    let registry = AdapterRegistry::new();
    assert!(registry.names().is_empty(), "fresh registry should have no frameworks");
}

#[test]
fn test_adapter_detect_no_project() {
    let registry = AdapterRegistry::new();
    let dir = std::env::temp_dir().join("klyron_adapter_test_nonexistent");
    let result = registry.detect(&dir);
    assert!(result.is_empty(), "detect on nonexistent dir should be empty");
}
