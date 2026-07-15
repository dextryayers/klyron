use crate::PmError;
use sha2::{Digest, Sha256, Sha512};
use std::path::Path;

/// Double-check all package hashes during install to ensure integrity.
/// Compares computed hashes against lockfile expectations.

#[derive(Debug, Clone)]
pub struct IntegrityVerifier;

impl IntegrityVerifier {
    pub fn verify_file_hash(path: &Path, expected: &str) -> Result<(), PmError> {
        let data = std::fs::read(path)
            .map_err(|e| PmError::IoError(format!("Cannot read {}: {e}", path.display())))?;

        let computed = if expected.starts_with("sha512-") {
            Self::sha512_hash(&data)
        } else if expected.starts_with("sha256-") || expected.len() == 64 {
            Self::sha256_hash(&data)
        } else if expected.starts_with("sha1-") {
            Self::sha1_hash(&data)
        } else {
            return Err(PmError::IntegrityMismatch {
                expected: expected.to_string(),
                actual: "unknown hash algorithm".to_string(),
            });
        };

        let expected_clean = expected.split('-').nth(1).unwrap_or(expected);
        if computed != expected_clean {
            return Err(PmError::IntegrityMismatch {
                expected: expected.to_string(),
                actual: format!("{computed}"),
            });
        }

        Ok(())
    }

    pub fn verify_multiple_hashes(path: &Path, hashes: &[String]) -> Result<(), PmError> {
        if hashes.is_empty() {
            return Err(PmError::IntegrityMismatch {
                expected: "at least one hash".to_string(),
                actual: "no hashes provided".to_string(),
            });
        }

        let data = std::fs::read(path)
            .map_err(|e| PmError::IoError(format!("Cannot read {}: {e}", path.display())))?;

        for hash_spec in hashes {
            let (algo, expected) = hash_spec.split_once('-').unwrap_or(("sha512", hash_spec));
            let computed = match algo {
                "sha512" => Self::sha512_hash(&data),
                "sha256" => Self::sha256_hash(&data),
                "sha1" => Self::sha1_hash(&data),
                _ => return Err(PmError::IntegrityMismatch {
                    expected: hash_spec.clone(),
                    actual: "unknown algorithm".to_string(),
                }),
            };
            if computed != expected {
                return Err(PmError::IntegrityMismatch {
                    expected: hash_spec.clone(),
                    actual: format!("{computed}"),
                });
            }
        }

        Ok(())
    }

    fn sha512_hash(data: &[u8]) -> String {
        let hash = Sha512::digest(data);
        hex::encode(hash)
    }

    fn sha256_hash(data: &[u8]) -> String {
        let hash = Sha256::digest(data);
        hex::encode(hash)
    }

    fn sha1_hash(data: &[u8]) -> String {
        let hash = sha1::Sha1::digest(data);
        hex::encode(hash)
    }

    pub fn compute_integrity(data: &[u8], algorithm: &str) -> String {
        match algorithm {
            "sha512" => {
                let hash = Sha512::digest(data);
                format!("sha512-{}", hex::encode(hash))
            }
            "sha256" => {
                let hash = Sha256::digest(data);
                format!("sha256-{}", hex::encode(hash))
            }
            _ => {
                let hash = Sha512::digest(data);
                format!("sha512-{}", hex::encode(hash))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha512_verification() {
        let dir = std::env::temp_dir().join("klyron_integrity_test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("test.bin");
        std::fs::write(&file, b"hello world").unwrap();

        let hash = IntegrityVerifier::compute_integrity(b"hello world", "sha512");
        assert!(IntegrityVerifier::verify_file_hash(&file, &hash).is_ok());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_sha256_verification() {
        let hash = IntegrityVerifier::compute_integrity(b"test data", "sha256");
        assert!(hash.starts_with("sha256-"));
        assert_eq!(hash.len(), 64 + 7);
    }

    #[test]
    fn test_wrong_hash_fails() {
        let dir = std::env::temp_dir().join("klyron_integrity_fail_test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("test.bin");
        std::fs::write(&file, b"hello world").unwrap();

        let result = IntegrityVerifier::verify_file_hash(&file, "sha512-abcdef");
        assert!(result.is_err());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_empty_file_hash() {
        let hash = IntegrityVerifier::compute_integrity(b"", "sha512");
        assert!(hash.starts_with("sha512-"));
    }
}
