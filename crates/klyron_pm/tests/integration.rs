use klyron_pm::lockfile::LockfilePackage;
use klyron_pm::KlyronLockfile;

#[test]
fn test_lockfile_new() {
    let lock = KlyronLockfile::new();
    assert!(lock.packages.is_empty());
}

#[test]
fn test_lockfile_add_package() {
    let mut lock = KlyronLockfile::new();
    lock.packages.insert(
        "lodash".to_string(),
        LockfilePackage {
            name: "lodash".to_string(),
            version: "4.17.21".to_string(),
            resolved: "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz".to_string(),
            integrity: String::new(),
            integrity_hashes: Vec::new(),
            signature: None,
            signer: None,
            dependencies: std::collections::HashMap::new(),
            optional_dependencies: std::collections::HashMap::new(),
            peer_dependencies: std::collections::HashMap::new(),
            bin: None,
            has_node_modules: false,
            install_time_ms: 0,
        },
    );
    assert_eq!(lock.packages.len(), 1);
    assert_eq!(lock.packages["lodash"].version, "4.17.21");
}

#[test]
fn test_lockfile_roundtrip() {
    let mut lock = KlyronLockfile::new();
    lock.packages.insert(
        "react".to_string(),
        LockfilePackage {
            name: "react".to_string(),
            version: "18.2.0".to_string(),
            resolved: "https://registry.npmjs.org/react/-/react-18.2.0.tgz".to_string(),
            integrity: String::new(),
            integrity_hashes: Vec::new(),
            signature: None,
            signer: None,
            dependencies: std::collections::HashMap::new(),
            optional_dependencies: std::collections::HashMap::new(),
            peer_dependencies: std::collections::HashMap::new(),
            bin: None,
            has_node_modules: false,
            install_time_ms: 0,
        },
    );
    let bytes = lock.to_bytes().unwrap();
    let restored = KlyronLockfile::from_bytes(&bytes).unwrap();
    assert_eq!(restored.packages.len(), 1);
    assert_eq!(restored.packages["react"].version, "18.2.0");
}
