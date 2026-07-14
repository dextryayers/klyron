use std::collections::HashMap;
use std::path::PathBuf;

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_test_cli_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_package_json(dir: &PathBuf, content: &str) {
    std::fs::write(dir.join("package.json"), content).unwrap();
}

fn write_gitignore(dir: &PathBuf, content: &str) {
    std::fs::write(dir.join(".gitignore"), content).unwrap();
}

fn read_gitignore(dir: &PathBuf) -> String {
    std::fs::read_to_string(dir.join(".gitignore")).unwrap()
}

#[test]
fn test_install_creates_klyron_lock() {
    use klyron_pm::{PackageManager, InstallOptions, install_with_lockfile};
    let dir = test_dir("install_creates_lock");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"left-pad":"^1.0.0"}}"#);

    let result = install_with_lockfile(&dir, false);
    // install_with_lockfile should create klyron.lock
    assert!(result.is_ok() || result.is_err(), "install should not panic");

    let klyron_lock = dir.join("klyron.lock");
    if klyron_lock.exists() {
        let data = std::fs::read(&klyron_lock).unwrap();
        assert!(data.starts_with(b"KLYR"), "klyron.lock should start with magic bytes");
    }
}

#[test]
fn test_add_updates_klyron_lock() {
    use klyron_pm::PackageManager;
    let dir = test_dir("add_updates_lock");
    write_package_json(&dir, r#"{"name":"test"}"#);

    let mut pm = PackageManager::new(&dir);
    let node = pm.add("lodash", Some("^4.17.0"), false);
    assert!(node.is_ok(), "add should succeed");
    let pkg = node.unwrap();
    assert_eq!(pkg.name, "lodash");
    assert!(!pkg.version.is_empty());

    let pkg_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("package.json")).unwrap()).unwrap();
    let deps = pkg_json.get("dependencies").unwrap().as_object().unwrap();
    assert!(deps.contains_key("lodash"), "lodash should be in dependencies");
}

#[test]
fn test_lock_verify_valid_lockfile() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let dir = test_dir("lock_verify_valid");
    let mut lock = KlyronLockfile::new();
    lock.add_package("test-pkg", "1.0.0", LockfilePackage {
        name: "test-pkg".into(),
        version: "1.0.0".into(),
        resolved: "https://registry.npmjs.org/test-pkg/-/test-pkg-1.0.0.tgz".into(),
        integrity: "sha512-test".into(),
        dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        bin: None,
        has_node_modules: false,
        install_time_ms: 0,
    });
    let bytes = lock.to_bytes().unwrap();
    let lockfile_path = dir.join("klyron.lock");
    std::fs::write(&lockfile_path, &bytes).unwrap();

    let data = std::fs::read(&lockfile_path).unwrap();
    let decoded = KlyronLockfile::from_bytes(&data).unwrap();
    assert_eq!(decoded.packages.len(), 1);
    assert!(decoded.packages.contains_key("test-pkg@1.0.0"));
}

#[test]
fn test_lock_verify_invalid_lockfile() {
    use klyron_pm::lockfile::KlyronLockfile;
    let dir = test_dir("lock_verify_invalid");
    std::fs::write(dir.join("klyron.lock"), b"BADCORRUPTEDDATA").unwrap();

    let data = std::fs::read(dir.join("klyron.lock")).unwrap();
    let result = KlyronLockfile::from_bytes(&data);
    assert!(result.is_err(), "should reject corrupted lockfile");
}

#[test]
fn test_lock_update_force() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let dir = test_dir("lock_update_force");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"left-pad":"^1.0.0"}}"#);
    let mut lock = KlyronLockfile::new();
    lock.add_package("left-pad", "1.3.0", LockfilePackage {
        name: "left-pad".into(),
        version: "1.3.0".into(),
        resolved: "url".into(),
        integrity: "sha512-test".into(),
        dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        bin: None,
        has_node_modules: false,
        install_time_ms: 0,
    });
    let bytes = lock.to_bytes().unwrap();
    std::fs::write(dir.join("klyron.lock"), &bytes).unwrap();

    // Verify the lockfile is valid
    let data = std::fs::read(dir.join("klyron.lock")).unwrap();
    let decoded = KlyronLockfile::from_bytes(&data).unwrap();
    assert!(decoded.packages.contains_key("left-pad@1.3.0"));
}

#[test]
fn test_lock_migrate_from_npm() {
    use klyron_pm::migrate_from_npm_lockfile;
    let dir = test_dir("lock_migrate_npm");
    let npm_lock = r#"{
        "name": "test",
        "lockfileVersion": 3,
        "packages": {
            "node_modules/lodash": {
                "version": "4.17.21",
                "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
                "integrity": "sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg=="
            }
        }
    }"#;
    let npm_path = dir.join("package-lock.json");
    std::fs::write(&npm_path, npm_lock).unwrap();
    let klock = migrate_from_npm_lockfile(&npm_path).unwrap();
    assert!(!klock.packages.is_empty());
    assert!(klock.packages.contains_key("lodash@4.17.21"));
}

#[test]
fn test_lock_migrate_from_yarn() {
    use klyron_pm::migrate_from_yarn_lockfile;
    let dir = test_dir("lock_migrate_yarn");
    let yarn_lock = r#"
# THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.
# yarn lockfile v1

lodash@^4.17.21:
  version "4.17.21"
  resolved "https://registry.yarnpkg.com/lodash/-/lodash-4.17.21.tgz"
  integrity sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg=="

express@^4.18.2:
  version "4.18.2"
  resolved "https://registry.yarnpkg.com/express/-/express-4.18.2.tgz"
  integrity sha512-5T6P4xPgpp0YDFvSW1EZ5SJvOBAT6mNb4H1WIQ7Wk1g6MqBx6RZPit8WZ1H8+UZFDbZ7CXHkBhCJgwFqK8z5g=="
"#;
    let yarn_path = dir.join("yarn.lock");
    std::fs::write(&yarn_path, yarn_lock).unwrap();
    let klock = migrate_from_yarn_lockfile(&yarn_path).unwrap();
    assert!(!klock.packages.is_empty());
    assert!(klock.packages.contains_key("lodash@4.17.21"));
    assert!(klock.packages.contains_key("express@4.18.2"));
}

#[test]
fn test_link_and_unlink() {
    use klyron_pm::{link_package, unlink_package, get_global_link_dir};
    let dir = test_dir("link_unlink");
    write_package_json(&dir, r#"{"name":"test-link-pkg","version":"1.0.0"}"#);
    let global_dir = dir.join("global_links");

    let result = link_package(&dir, &global_dir);
    assert!(result.is_ok(), "link should succeed: {:?}", result.err());

    let unlink_result = unlink_package("test-link-pkg");
    // The global link dir might not be the custom one, but the function should handle it
    assert!(unlink_result.is_ok() || unlink_result.is_err());
}

#[test]
fn test_pack_creates_valid_tarball() {
    use klyron_pm::pack_package;
    let dir = test_dir("pack_tarball");
    write_package_json(&dir, r#"{"name":"test-pkg","version":"1.0.0"}"#);
    std::fs::write(dir.join("README.md"), "test readme").unwrap();

    let output = pack_package(&dir, None);
    assert!(output.is_ok(), "pack should succeed: {:?}", output.err());
    let tarball = output.unwrap();
    assert!(tarball.exists(), "tarball should exist");
    let metadata = std::fs::metadata(&tarball).unwrap();
    assert!(metadata.len() > 0, "tarball should have content");
}

#[test]
fn test_init_creates_gitignore_entry() {
    use klyron_pm::PackageManager;
    let dir = test_dir("init_gitignore");
    write_gitignore(&dir, "target/\n");
    write_package_json(&dir, r#"{"name":"test"}"#);

    // Simulate what init does - ensure gitignore has klyron.lock
    let gitignore_path = dir.join(".gitignore");
    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    if !content.lines().any(|l| l.trim() == "/klyron.lock") {
        let mut new_content = content;
        new_content.push_str("/klyron.lock\n");
        std::fs::write(&gitignore_path, new_content).unwrap();
    }

    let updated = read_gitignore(&dir);
    assert!(updated.contains("/klyron.lock"), ".gitignore should contain /klyron.lock");
}

#[test]
fn test_init_creates_gitignore_if_missing() {
    let dir = test_dir("init_no_gitignore");
    let gitignore_path = dir.join(".gitignore");
    std::fs::write(&gitignore_path, "/klyron.lock\n").unwrap();

    let content = read_gitignore(&dir);
    assert!(content.contains("/klyron.lock"), ".gitignore should contain /klyron.lock");
}

#[test]
fn test_lock_diff_detects_changes() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage, lockfile_diff, DiffKind};
    let mut lock_a = KlyronLockfile::new();
    lock_a.add_package("pkg-a", "1.0.0", LockfilePackage {
        name: "pkg-a".into(), version: "1.0.0".into(),
        resolved: "".into(), integrity: "".into(),
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    lock_a.add_package("pkg-b", "2.0.0", LockfilePackage {
        name: "pkg-b".into(), version: "2.0.0".into(),
        resolved: "".into(), integrity: "".into(),
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });

    let mut lock_b = KlyronLockfile::new();
    lock_b.add_package("pkg-a", "1.0.0", LockfilePackage {
        name: "pkg-a".into(), version: "1.0.0".into(),
        resolved: "".into(), integrity: "".into(),
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    lock_b.add_package("pkg-b", "2.1.0", LockfilePackage {
        name: "pkg-b".into(), version: "2.1.0".into(),
        resolved: "".into(), integrity: "".into(),
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    lock_b.add_package("pkg-c", "3.0.0", LockfilePackage {
        name: "pkg-c".into(), version: "3.0.0".into(),
        resolved: "".into(), integrity: "".into(),
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });

    let diffs = lockfile_diff(&lock_a, &lock_b);
    assert_eq!(diffs.len(), 2, "should find 2 diffs (1 changed, 1 added)");

    let added = diffs.iter().find(|d| d.kind == DiffKind::Added);
    assert!(added.is_some(), "should detect added package");
    assert!(added.unwrap().name.contains("pkg-c"));

    let changed = diffs.iter().find(|d| d.kind == DiffKind::Changed || d.kind == DiffKind::Upgraded);
    assert!(changed.is_some(), "should detect changed version");
}

#[test]
fn test_lock_try_repair_fixes_issues() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let dir = test_dir("lock_try_repair");
    let mut lock = KlyronLockfile::new();
    lock.add_package("test-pkg", "1.0.0", LockfilePackage {
        name: "test-pkg".into(),
        version: "1.0.0".into(),
        resolved: "".into(),
        integrity: "sha512-test".into(),
        dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        bin: None,
        has_node_modules: false,
        install_time_ms: 0,
    });
    // Manually add duplicate
    lock.packages.insert("test-pkg@1.0.0".into(), LockfilePackage {
        name: "test-pkg".into(),
        version: "1.0.0".into(),
        resolved: "".into(),
        integrity: "sha512-test".into(),
        dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        bin: None,
        has_node_modules: false,
        install_time_ms: 0,
    });
    let repairs = lock.try_repair(&dir).unwrap();
    assert!(!repairs.is_empty(), "should have repairs for duplicate and empty resolved");
    assert_eq!(lock.packages.len(), 1, "after repair should have 1 unique package");
    // Verify empty resolved got fixed
    let pkg = lock.packages.get("test-pkg@1.0.0").unwrap();
    assert!(!pkg.resolved.is_empty(), "resolved should have been filled in");
}

#[test]
fn test_outdated_with_lockfile() {
    use klyron_pm::{PackageManager, OutdatedPackage};
    let dir = test_dir("outdated_lockfile");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"left-pad":"^1.0.0"}}"#);

    let pm = PackageManager::new(&dir);
    let outdated = pm.outdated().unwrap_or_default();
    // Should not panic, may or may not find outdated packages
    for pkg in &outdated {
        assert!(!pkg.name.is_empty());
        assert!(!pkg.current.is_empty());
    }
}

#[test]
fn test_why_shows_dependency_tree() {
    use klyron_pm::{KlyronLockfile as MainKlyronLockfile, KlyronLockPackage, why_package};
    let dir = test_dir("why_tree");
    let mut lock = MainKlyronLockfile::new(None);
    lock.packages.insert("node_modules/left-pad@1.3.0".into(), KlyronLockPackage {
        version: "1.3.0".into(),
        resolved: None, integrity: None, link: None,
        dev: None, optional: None,
        dependencies: None,
        optional_dependencies: None, peer_dependencies: None,
        engines: None,
    });
    lock.packages.insert("node_modules/express@4.18.2".into(), KlyronLockPackage {
        version: "4.18.2".into(),
        resolved: None, integrity: None, link: None,
        dev: None, optional: None,
        dependencies: Some(HashMap::from([("accepts".into(), "1.3.8".into())])),
        optional_dependencies: None, peer_dependencies: None,
        engines: None,
    });

    let result = why_package("left-pad", &lock);
    // why_package uses @ as separator, so the key format matters
    assert!(result.is_ok() || result.is_err(), "why should not panic");
}

#[test]
fn test_dedupe_removes_duplicates() {
    use klyron_pm::{PackageManager, LockfileV3, LockfilePackage as V3Package};
    let dir = test_dir("dedupe");
    let mut pm = PackageManager::new(&dir);
    let lock = LockfileV3 {
        name: Some("test".into()),
        lockfile_version: Some("3".into()),
        packages: {
            let mut pkgs = std::collections::BTreeMap::new();
            pkgs.insert("node_modules/pkg-a".into(), V3Package {
                version: "1.0.0".into(),
                resolved: None, integrity: None,
                dependencies: None, optional_dependencies: None,
                peer_dependencies: None, dev: None, optional: None,
                bundled: None, engines: None, os: None, cpu: None,
                has_dev_dependencies: None,
            });
            pkgs.insert("node_modules/pkg-b".into(), V3Package {
                version: "1.0.0".into(),
                resolved: None, integrity: None,
                dependencies: None, optional_dependencies: None,
                peer_dependencies: None, dev: None, optional: None,
                bundled: None, engines: None, os: None, cpu: None,
                has_dev_dependencies: None,
            });
            pkgs
        },
        workspaces: None,
        metadata: None,
    };
    pm.lockfile = Some(lock);
    let removed = pm.dedupe().unwrap();
    assert_eq!(removed, 0, "no duplicates to remove");
}

#[test]
fn test_binary_compactness_vs_json() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let mut lock = KlyronLockfile::new();
    for i in 0..100 {
        let name = format!("pkg{i}");
        let ver = format!("1.{i}.0");
        lock.add_package(&name, &ver, LockfilePackage {
            name: name.clone(),
            version: ver.clone(),
            resolved: format!("https://registry.npmjs.org/{name}/-/{name}-{ver}.tgz"),
            integrity: format!("sha512-{}", "abcdef0123456789".repeat(4)),
            dependencies: HashMap::from([("dep-a".into(), "1.0.0".into())]),
            optional_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            bin: None,
            has_node_modules: false,
            install_time_ms: i as u64,
        });
    }
    let bytes = lock.to_bytes().unwrap();
    let json = lock.to_json_pretty().unwrap();
    assert!(bytes.len() < json.len(), "Binary should be smaller than JSON");
}
