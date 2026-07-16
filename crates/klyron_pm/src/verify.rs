use crate::PmError;
use sha2::{Digest, Sha512};
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_verify_integrity_compute_hash() {
        let data = b"test package data";
        let mut hasher = Sha512::new();
        hasher.update(data);
        let hash = format!("sha512-{}", hex::encode(hasher.finalize()));
        assert!(hash.starts_with("sha512-"));
        assert_eq!(hash.len(), 128 + 7); // "sha512-" + 128 hex chars
    }

    #[test]
    fn test_verify_integrity_mismatch() {
        let data = b"original data";
        let mut hasher = Sha512::new();
        hasher.update(data);
        let hash = format!("sha512-{}", hex::encode(hasher.finalize()));

        let wrong_data = b"tampered data";
        let mut wrong_hasher = Sha512::new();
        wrong_hasher.update(wrong_data);
        let wrong_hash = format!("sha512-{}", hex::encode(wrong_hasher.finalize()));

        assert_ne!(hash, wrong_hash);
    }

    #[test]
    fn test_pack_dry_run_no_package_json() {
        let tmp = std::env::temp_dir().join("klyron_test_verify");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let result = pack_dry_run(&tmp);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_pack_dry_run_with_package_json() {
        let tmp = std::env::temp_dir().join("klyron_test_verify_pkg");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let mut file = std::fs::File::create(tmp.join("package.json")).unwrap();
        file.write_all(b"{\"name\":\"test\"}").unwrap();
        std::fs::create_dir_all(tmp.join("src")).unwrap();
        std::fs::write(tmp.join("src").join("index.js"), "module.exports = {};").unwrap();
        let result = pack_dry_run(&tmp).unwrap();
        assert!(result.contains(&"package/src/index.js".to_string()));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_pack_dry_run_excludes_dot_git() {
        let tmp = std::env::temp_dir().join("klyron_test_verify_excl");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("package.json"), "{}").unwrap();
        std::fs::create_dir_all(tmp.join(".git")).unwrap();
        std::fs::write(tmp.join(".git").join("config"), "dummy").unwrap();
        let result = pack_dry_run(&tmp).unwrap();
        assert!(!result.iter().any(|f| f.contains(".git")));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_pack_dry_run_excludes_node_modules() {
        let tmp = std::env::temp_dir().join("klyron_test_verify_nm");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("package.json"), "{}").unwrap();
        std::fs::create_dir_all(tmp.join("node_modules").join("dep")).unwrap();
        std::fs::write(tmp.join("node_modules").join("dep").join("index.js"), "").unwrap();
        let result = pack_dry_run(&tmp).unwrap();
        assert!(!result.iter().any(|f| f.contains("node_modules")));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

pub fn verify_integrity(tarball_path: &Path, expected_hash: &str) -> Result<bool, PmError> {
    let data = std::fs::read(tarball_path)?;
    let mut hasher = Sha512::new();
    hasher.update(&data);
    let actual = format!("sha512-{}", hex::encode(hasher.finalize()));
    Ok(actual == expected_hash)
}

pub fn verify_signature(tarball_path: &Path, sig_path: &Path, pubkey_path: &Path) -> Result<bool, PmError> {
    let tarball = std::fs::read(tarball_path)?;
    let signature = std::fs::read(sig_path)?;
    let pem = std::fs::read_to_string(pubkey_path)?;

    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    use ed25519_dalek::pkcs8::DecodePublicKey;

    if signature.len() != 64 {
        return Ok(false);
    }
    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(&signature);
    let sig = Signature::from_bytes(&sig_bytes);

    let public = match VerifyingKey::from_public_key_pem(&pem) {
        Ok(k) => k,
        Err(_) => return Ok(false),
    };

    Ok(public.verify(&tarball, &sig).is_ok())
}

pub fn pack_dry_run(package_dir: &Path) -> Result<Vec<String>, PmError> {
    let pkg_json = package_dir.join("package.json");
    if !pkg_json.exists() {
        return Err(PmError::PackageNotFound("package.json not found".into()));
    }

    let mut files = Vec::new();
    let exclude = ["node_modules", ".git", "target", "test", "__tests__", "tests", ".klyron"];

    for entry in walkdir::WalkDir::new(package_dir).into_iter().filter_map(|e| e.ok()) {
        let relative = entry.path().strip_prefix(package_dir).unwrap_or(entry.path());
        let rel_str = relative.to_string_lossy();

        if rel_str == "." {
            continue;
        }

        let should_exclude = exclude.iter().any(|e| {
            rel_str == *e || rel_str.starts_with(&format!("{e}/"))
        });
        if should_exclude {
            continue;
        }

        if entry.file_type().is_file() {
            files.push(format!("package/{rel_str}"));
        } else if entry.file_type().is_dir() {
            files.push(format!("package/{rel_str}/"));
        }
    }

    files.sort();
    Ok(files)
}
