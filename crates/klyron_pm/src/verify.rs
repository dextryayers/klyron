use crate::PmError;
use sha2::{Digest, Sha512};
use std::path::Path;

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

    use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};
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
