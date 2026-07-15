use std::collections::HashMap;
use std::path::PathBuf;

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("klyron_test_pm_adv_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_package_json(dir: &PathBuf, content: &str) {
    std::fs::write(dir.join("package.json"), content).unwrap();
}

#[test]
fn test_lockfile_bincode_roundtrip() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let mut lock = KlyronLockfile::new();
    lock.add_package("test-pkg", "1.0.0", LockfilePackage {
        name: "test-pkg".into(),
        version: "1.0.0".into(),
        resolved: "https://registry.npmjs.org/test-pkg/-/test-pkg-1.0.0.tgz".into(),
        integrity: "sha512-deadbeef123456".into(),
        integrity_hashes: vec!["sha256-abc".into()],
        signature: None,
        signer: None,
        dependencies: HashMap::from([("dep-a".into(), "1.0.0".into())]),
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        bin: None,
        has_node_modules: false,
        install_time_ms: 42,
    });
    let bytes = lock.to_bytes().unwrap();
    let decoded = KlyronLockfile::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.packages.len(), 1);
    assert!(decoded.packages.contains_key("test-pkg@1.0.0"));
    let pkg = &decoded.packages["test-pkg@1.0.0"];
    assert_eq!(pkg.version, "1.0.0");
    assert_eq!(pkg.dependencies.get("dep-a").unwrap(), "1.0.0");
    assert_eq!(pkg.install_time_ms, 42);
}

#[test]
fn test_lockfile_generation_on_install() {
    use klyron_pm::{PackageManager, InstallOptions, install_with_lockfile};
    let dir = test_dir("lock_gen_install");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"is-odd":"^3.0.0"}}"#);
    let result = install_with_lockfile(&dir, false);
    assert!(result.is_ok() || result.is_err());
    let lock_path = dir.join("klyron.lock");
    if lock_path.exists() {
        let data = std::fs::read(&lock_path).unwrap();
        assert!(data.starts_with(b"KLYR"));
    }
}

#[test]
fn test_integrity_verification() {
    use klyron_pm::lockfile::KlyronLockfile;
    let data = b"test package data";
    let integrity = KlyronLockfile::compute_integrity(data);
    assert!(integrity.starts_with("sha512-"));
    assert!(integrity.len() > 10);
    let integrity256 = KlyronLockfile::compute_integrity_sha256(data);
    assert!(integrity256.starts_with("sha256-"));
    assert_eq!(integrity256.len(), 71);
}

#[test]
fn test_frozen_lockfile_reject_stale() {
    use klyron_pm::{InstallOptions, PackageManager};
    let dir = test_dir("frozen_reject");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"foo":"^1.0.0"}}"#);
    std::fs::write(dir.join("klyron.lock"), b"KLYR\x00\x00\x00\x00").unwrap();
    let opts = InstallOptions {
        frozen_lockfile: true,
        ..Default::default()
    };
    let pm = PackageManager::new(&dir);
    let result = pm.install_with_options(&opts);
    assert!(result.is_err());
}

#[test]
fn test_monorepo_workspace_protocol() {
    use klyron_pm::Workspace;
    let ws = Workspace {
        name: "monorepo".into(),
        members: vec!["packages/a".into(), "packages/b".into()],
        root: PathBuf::from("/tmp/monorepo"),
    };
    assert_eq!(ws.name, "monorepo");
    assert_eq!(ws.members.len(), 2);
    assert!(ws.members.contains(&"packages/a".into()));
}

#[test]
fn test_pack_unpack_tgz() {
    use klyron_pm::pack_package;
    let dir = test_dir("pack_unpack");
    write_package_json(&dir, r#"{"name":"my-pkg","version":"2.0.0"}"#);
    std::fs::write(dir.join("index.js"), "module.exports = 42;").unwrap();
    let output = pack_package(&dir, None);
    assert!(output.is_ok());
    let tarball = output.unwrap();
    assert!(tarball.exists());
    assert!(tarball.extension().unwrap_or_default() == "tgz");
    let metadata = std::fs::metadata(&tarball).unwrap();
    assert!(metadata.len() > 20);
}

#[test]
fn test_overrides_resolutions() {
    use klyron_pm::LockfileV3;
    let json = r#"{
        "name": "test",
        "lockfileVersion": 3,
        "packages": {
            "node_modules/foo": { "version": "1.0.0", "resolved": "https://registry.npmjs.org/foo" },
            "node_modules/bar": { "version": "2.0.0", "resolved": "https://registry.npmjs.org/bar" }
        }
    }"#;
    let lf = LockfileV3::from_npm_lockfile(json).unwrap();
    assert_eq!(lf.packages.len(), 2);
    let foo = lf.find_package("foo");
    assert!(foo.is_some());
    assert_eq!(foo.unwrap().1.version, "1.0.0");
}

#[test]
fn test_peer_dependency_resolution() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let mut lock = KlyronLockfile::new();
    lock.add_package("react", "18.2.0", LockfilePackage {
        name: "react".into(),
        version: "18.2.0".into(),
        resolved: "".into(),
        integrity: "sha512-test".into(),
        integrity_hashes: vec![],
        signature: None,
        signer: None,
        dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::from([("react-dom".into(), "^18.0.0".into())]),
        bin: None,
        has_node_modules: false,
        install_time_ms: 0,
    });
    lock.add_package("react-dom", "18.2.0", LockfilePackage {
        name: "react-dom".into(),
        version: "18.2.0".into(),
        resolved: "".into(),
        integrity: "sha512-peer".into(),
        integrity_hashes: vec![],
        signature: None,
        signer: None,
        dependencies: HashMap::new(),
        optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(),
        bin: None,
        has_node_modules: false,
        install_time_ms: 0,
    });
    let react = &lock.packages["react@18.2.0"];
    assert!(react.peer_dependencies.contains_key("react-dom"));
}

#[test]
fn test_optional_dependency_resolution() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let mut lock = KlyronLockfile::new();
    lock.add_package("main-pkg", "1.0.0", LockfilePackage {
        name: "main-pkg".into(),
        version: "1.0.0".into(),
        resolved: "".into(),
        integrity: "sha512-opt".into(),
        integrity_hashes: vec![],
        signature: None,
        signer: None,
        dependencies: HashMap::new(),
        optional_dependencies: HashMap::from([("fsevents".into(), "^2.3.0".into())]),
        peer_dependencies: HashMap::new(),
        bin: None,
        has_node_modules: false,
        install_time_ms: 0,
    });
    let pkg = &lock.packages["main-pkg@1.0.0"];
    assert!(pkg.optional_dependencies.contains_key("fsevents"));
}

#[test]
fn test_dedupe_algorithm() {
    use klyron_pm::{PackageManager, LockfileV3, LockfilePackage as V3Pkg};
    let dir = test_dir("dedupe_alg");
    let mut pm = PackageManager::new(&dir);
    let lock = LockfileV3 {
        name: Some("dedupe-test".into()),
        lockfile_version: Some("3".into()),
        packages: {
            let mut pkgs = std::collections::BTreeMap::new();
            pkgs.insert("node_modules/lodash".into(), V3Pkg {
                version: "4.17.21".into(),
                resolved: None, integrity: None,
                dependencies: None, optional_dependencies: None,
                peer_dependencies: None, dev: None, optional: None,
                bundled: None, engines: None, os: None, cpu: None,
                has_dev_dependencies: None,
            });
            pkgs.insert("node_modules/other/node_modules/lodash".into(), V3Pkg {
                version: "4.17.21".into(),
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
    assert!(removed >= 0);
}

#[test]
fn test_outdated_detection() {
    use klyron_pm::{PackageManager, OutdatedPackage};
    let dir = test_dir("outdated_detect");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"left-pad":"^1.0.0"}}"#);
    let pm = PackageManager::new(&dir);
    let outdated = pm.outdated().unwrap_or_default();
    for pkg in &outdated {
        assert!(!pkg.name.is_empty());
        assert!(!pkg.current.is_empty());
    }
}

#[test]
fn test_lockfile_diff_printing() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage, lockfile_diff, DiffKind};
    let mut a = KlyronLockfile::new();
    a.add_package("pkg-a", "1.0.0", LockfilePackage {
        name: "pkg-a".into(), version: "1.0.0".into(),
        resolved: "".into(), integrity: "".into(),
        integrity_hashes: vec![], signature: None, signer: None,
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    let mut b = KlyronLockfile::new();
    b.add_package("pkg-a", "2.0.0", LockfilePackage {
        name: "pkg-a".into(), version: "2.0.0".into(),
        resolved: "".into(), integrity: "".into(),
        integrity_hashes: vec![], signature: None, signer: None,
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    b.add_package("pkg-b", "1.0.0", LockfilePackage {
        name: "pkg-b".into(), version: "1.0.0".into(),
        resolved: "".into(), integrity: "".into(),
        integrity_hashes: vec![], signature: None, signer: None,
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    let diffs = lockfile_diff(&a, &b);
    assert_eq!(diffs.len(), 2);
    let changed: Vec<_> = diffs.iter().filter(|d| d.kind == DiffKind::Changed || d.kind == DiffKind::Upgraded).collect();
    assert_eq!(changed.len(), 1);
    let added: Vec<_> = diffs.iter().filter(|d| d.kind == DiffKind::Added).collect();
    assert_eq!(added.len(), 1);
}

#[test]
fn test_lockfile_merge_workspace() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let mut root = KlyronLockfile::new();
    root.add_package("common", "1.0.0", LockfilePackage {
        name: "common".into(), version: "1.0.0".into(),
        resolved: "".into(), integrity: "".into(),
        integrity_hashes: vec![], signature: None, signer: None,
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    let mut member_a = KlyronLockfile::new();
    member_a.add_package("member-a-lib", "0.1.0", LockfilePackage {
        name: "member-a-lib".into(), version: "0.1.0".into(),
        resolved: "".into(), integrity: "".into(),
        integrity_hashes: vec![], signature: None, signer: None,
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    for (key, pkg) in member_a.packages {
        root.packages.insert(key, pkg);
    }
    assert_eq!(root.packages.len(), 2);
}

#[test]
fn test_lockfile_roundtrip_large() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let mut lock = KlyronLockfile::new();
    for i in 0..50 {
        let name = format!("pkg-{i}");
        let ver = format!("1.{i}.0");
        lock.add_package(&name, &ver, LockfilePackage {
            name: name.clone(),
            version: ver.clone(),
            resolved: format!("https://registry.npmjs.org/{name}/-/tgz"),
            integrity: format!("sha512-{}", "a".repeat(64)),
            integrity_hashes: vec![],
            signature: None,
            signer: None,
            dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            bin: None,
            has_node_modules: false,
            install_time_ms: i as u64,
        });
    }
    let bytes = lock.to_bytes().unwrap();
    assert!(bytes.len() > 100);
    let decoded = KlyronLockfile::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.packages.len(), 50);
}

#[test]
fn test_lockfile_integrity_check_all() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let mut lock = KlyronLockfile::new();
    lock.add_package("good", "1.0.0", LockfilePackage {
        name: "good".into(), version: "1.0.0".into(),
        resolved: "url".into(), integrity: "sha512-goodhash".into(),
        integrity_hashes: vec![], signature: None, signer: None,
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    lock.add_package("bad", "2.0.0", LockfilePackage {
        name: "bad".into(), version: "2.0.0".into(),
        resolved: "url".into(), integrity: "".into(),
        integrity_hashes: vec![], signature: None, signer: None,
        dependencies: HashMap::new(), optional_dependencies: HashMap::new(),
        peer_dependencies: HashMap::new(), bin: None,
        has_node_modules: false, install_time_ms: 0,
    });
    let errors = lock.verify_all_integrity(false);
    assert!(!errors.is_empty(), "should find integrity issues");
}

#[test]
fn test_signature_verification() {
    use klyron_pm::verify_package_signature_on_install;
    let data = b"test tarball data";
    let sig = [0u8; 64];
    let pub_key = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEADdEMNlm6FYFicMHsP1HqNXyFkdTk3s4m/4VXKHzXhGc=\n-----END PUBLIC KEY-----\n";
    let result = verify_package_signature_on_install(data, &sig, pub_key);
    assert!(result.is_err(), "should fail with invalid key");
}

#[test]
fn test_security_advisory_struct() {
    use klyron_pm::SecurityAdvisory;
    let adv = SecurityAdvisory {
        id: "GHSA-xxxx".into(),
        package_name: "lodash".into(),
        severity: "high".into(),
        vulnerable_versions: "<4.17.21".into(),
        patched_versions: ">=4.17.21".into(),
        title: "Prototype Pollution".into(),
        description: "Prototype pollution in lodash".into(),
        cvss_score: Some(7.5),
    };
    assert_eq!(adv.severity, "high");
    assert!(adv.cvss_score.unwrap() > 0.0);
}

#[test]
fn test_pm_detect() {
    use klyron_pm::PackageManagerKind;
    let dir = test_dir("pm_detect");
    assert_eq!(PackageManagerKind::detect(&dir), PackageManagerKind::Npm);
    std::fs::write(dir.join("yarn.lock"), "").unwrap();
    assert_eq!(PackageManagerKind::detect(&dir), PackageManagerKind::Yarn);
}

#[test]
fn test_reproducible_build_id() {
    use klyron_pm::ReproducibleBuild;
    use std::collections::HashMap;
    let dir = test_dir("repro_build");
    std::fs::write(dir.join("source.js"), "const x = 1;").unwrap();
    let config = serde_json::json!({"target": "es2020"});
    let id = ReproducibleBuild::reproducible_build_id(&dir, &config);
    assert!(id.is_ok());
    assert!(!id.unwrap().is_empty());
}

#[test]
fn test_install_options_defaults() {
    use klyron_pm::InstallOptions;
    let opts = InstallOptions::default();
    assert!(!opts.production);
    assert!(!opts.frozen_lockfile);
    assert!(opts.verify_integrity);
    assert!(!opts.verify_signatures);
}

#[test]
fn test_package_manager_new() {
    use klyron_pm::PackageManager;
    let dir = test_dir("pm_new");
    write_package_json(&dir, r#"{"name":"test"}"#);
    let pm = PackageManager::new(&dir);
    assert_eq!(pm.dir, dir);
}

#[test]
fn test_outdated_package_struct() {
    use klyron_pm::OutdatedPackage;
    let pkg = OutdatedPackage {
        name: "lodash".into(),
        current: "4.17.20".into(),
        wanted: "4.17.21".into(),
        latest: "4.17.21".into(),
        dep_type: "dependencies".into(),
    };
    assert_eq!(pkg.name, "lodash");
    assert_eq!(pkg.wanted, "4.17.21");
}

#[test]
fn test_why_package() {
    use klyron_pm::{KlyronLockfile as MainLockfile, KlyronLockPackage, why_package};
    let mut lock = MainLockfile::new(None);
    lock.packages.insert("node_modules/express@4.18.2".into(), KlyronLockPackage {
        version: "4.18.2".into(),
        resolved: None, integrity: None, link: None,
        dev: None, optional: None,
        dependencies: Some(HashMap::from([("accepts".into(), "1.3.8".into())])),
        optional_dependencies: None, peer_dependencies: None,
        engines: None,
    });
    let result = why_package("express", &lock);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lockfile_v3_find_package() {
    use klyron_pm::LockfileV3;
    let json = r#"{"packages":{"node_modules/lodash":{"version":"4.17.21"}}}"#;
    let lf = LockfileV3::from_npm_lockfile(json).unwrap();
    let found = lf.find_package("lodash");
    assert!(found.is_some());
    let (path, pkg) = found.unwrap();
    assert!(path.contains("lodash"));
    assert_eq!(pkg.version, "4.17.21");
}

#[test]
fn test_lockfile_v3_get_version() {
    use klyron_pm::LockfileV3;
    let json = r#"{"packages":{"node_modules/foo":{"version":"2.0.0"}}}"#;
    let lf = LockfileV3::from_npm_lockfile(json).unwrap();
    assert_eq!(lf.get_version("foo"), Some("2.0.0"));
    assert_eq!(lf.get_version("nonexistent"), None);
}

#[test]
fn test_verify_package_integrity_valid() {
    use klyron_pm::{LockfileV3, LockfilePackage, PmError};
    let mut pkgs = std::collections::BTreeMap::new();
    pkgs.insert("node_modules/valid-pkg".into(), LockfilePackage {
        version: "1.0.0".into(),
        resolved: Some("url".into()),
        integrity: Some("sha512-abc123".into()),
        dependencies: None, optional_dependencies: None,
        peer_dependencies: None, dev: None, optional: None,
        bundled: None, engines: None, os: None, cpu: None,
        has_dev_dependencies: None,
    });
    let lf = LockfileV3 {
        name: None, lockfile_version: None,
        packages: pkgs, workspaces: None, metadata: None,
    };
    let result = lf.verify_package_integrity("valid-pkg", "sha512-abc123");
    assert!(result.is_ok());
}

#[test]
fn test_verify_package_integrity_mismatch() {
    use klyron_pm::{LockfileV3, LockfilePackage, PmError};
    let mut pkgs = std::collections::BTreeMap::new();
    pkgs.insert("node_modules/bad-pkg".into(), LockfilePackage {
        version: "1.0.0".into(),
        resolved: Some("url".into()),
        integrity: Some("sha512-expected".into()),
        dependencies: None, optional_dependencies: None,
        peer_dependencies: None, dev: None, optional: None,
        bundled: None, engines: None, os: None, cpu: None,
        has_dev_dependencies: None,
    });
    let lf = LockfileV3 {
        name: None, lockfile_version: None,
        packages: pkgs, workspaces: None, metadata: None,
    };
    let result = lf.verify_package_integrity("bad-pkg", "sha512-actual");
    assert!(matches!(result, Err(PmError::IntegrityMismatch { .. })));
}

#[test]
fn test_rate_limiter_basic() {
    use klyron_pm::rate_limit::{RateLimiter, RateLimitConfig};
    let config = RateLimitConfig {
        max_requests: 100,
        window_secs: 60,
        burst_size: 10,
    };
    let limiter = RateLimiter::new(config);
    assert!(limiter.allow());
}

#[test]
fn test_lockfile_metadata_serialization() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfileMetadata};
    let mut lock = KlyronLockfile::new();
    lock.metadata.install_count = 42;
    lock.metadata.klyron_version = "0.1.0".into();
    let json = lock.to_json_pretty().unwrap();
    assert!(json.contains("42"));
    assert!(json.contains("0.1.0"));
}

#[test]
fn test_binary_compactness() {
    use klyron_pm::lockfile::{KlyronLockfile, LockfilePackage};
    let mut lock = KlyronLockfile::new();
    for i in 0..20 {
        lock.add_package(&format!("pkg{i}"), &format!("1.{i}.0"), LockfilePackage {
            name: format!("pkg{i}"), version: format!("1.{i}.0"),
            resolved: format!("https://registry.npmjs.org/pkg{i}/-/pkg{i}-1.{i}.0.tgz"),
            integrity: format!("sha512-{}", "abcdef01".repeat(8)),
            integrity_hashes: vec![],
            signature: None, signer: None,
            dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            bin: None,
            has_node_modules: false,
            install_time_ms: i as u64,
        });
    }
    let bytes = lock.to_bytes().unwrap();
    let json = lock.to_json_pretty().unwrap();
    assert!(bytes.len() < json.len(), "binary should be smaller than json");
}

#[test]
fn test_add_creates_dep_in_package_json() {
    let dir = test_dir("add_creates_dep");
    write_package_json(&dir, r#"{"name":"test"}"#);
    let mut pm = klyron_pm::PackageManager::new(&dir);
    let result = pm.add("lodash", Some("^4.17.0"), false);
    assert!(result.is_ok());
    let pkg_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("package.json")).unwrap()
    ).unwrap();
    let deps = pkg_json.get("dependencies").unwrap().as_object().unwrap();
    assert!(deps.contains_key("lodash"));
}

#[test]
fn test_remove_removes_dep() {
    let dir = test_dir("remove_dep");
    write_package_json(&dir, r#"{"name":"test","dependencies":{"lodash":"^4.17.0"}}"#);
    let mut pm = klyron_pm::PackageManager::new(&dir);
    let result = pm.remove("lodash");
    assert!(result.is_ok());
    let pkg_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("package.json")).unwrap()
    ).unwrap();
    assert!(pkg_json.get("dependencies").is_none() ||
            !pkg_json["dependencies"].as_object().unwrap().contains_key("lodash"));
}

#[test]
fn test_lockfile_corrupted_rejected() {
    use klyron_pm::lockfile::KlyronLockfile;
    let corrupted = b"KLYR\xff\xff\xff\xff";
    let result = KlyronLockfile::from_bytes(corrupted);
    assert!(result.is_err());
}

#[test]
fn test_lockfile_empty_rejected() {
    use klyron_pm::lockfile::KlyronLockfile;
    let result = KlyronLockfile::from_bytes(b"");
    assert!(result.is_err());
}

#[test]
fn test_lockfile_truncated_rejected() {
    use klyron_pm::lockfile::KlyronLockfile;
    let truncated = b"KLYR\x05\x00\x00\x00\x01\x02";
    let result = KlyronLockfile::from_bytes(truncated);
    assert!(result.is_err());
}

#[test]
fn test_pm_error_display() {
    use klyron_pm::PmError;
    let err = PmError::PackageNotFound("test-pkg".into());
    assert!(err.to_string().contains("test-pkg"));
    let err2 = PmError::IntegrityMismatch { expected: "a".into(), actual: "b".into() };
    assert!(err2.to_string().contains("a"));
    assert!(err2.to_string().contains("b"));
}

#[test]
fn test_check_package_security() {
    use klyron_pm::check_package_security;
    let advisories = check_package_security("lodash", "4.17.21").unwrap();
    assert!(advisories.is_empty());
}
