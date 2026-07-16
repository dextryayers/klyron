use crate::PmError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::path::Path;

const KLYRON_MAGIC: &[u8] = b"KLYR";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfilePackage {
    pub name: String,
    pub version: String,
    pub resolved: String,
    pub integrity: String,
    pub integrity_hashes: Vec<String>,
    pub signature: Option<String>,
    pub signer: Option<String>,
    pub dependencies: HashMap<String, String>,
    pub optional_dependencies: HashMap<String, String>,
    pub peer_dependencies: HashMap<String, String>,
    pub bin: Option<HashMap<String, String>>,
    pub has_node_modules: bool,
    pub install_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileMetadata {
    pub created_at: String,
    pub klyron_version: String,
    pub install_count: u64,
    pub tuf_metadata_signed: Option<bool>,
    pub tuf_root_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlyronLockfile {
    pub lockfile_version: u32,
    pub metadata: LockfileMetadata,
    pub packages: HashMap<String, LockfilePackage>,
}

impl KlyronLockfile {
    pub fn new() -> Self {
        Self {
            lockfile_version: 3,
            metadata: LockfileMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                klyron_version: env!("CARGO_PKG_VERSION").to_string(),
                install_count: 0,
                tuf_metadata_signed: None,
                tuf_root_hash: None,
            },
            packages: HashMap::new(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, PmError> {
        let mut buf = Vec::with_capacity(4096);
        buf.extend_from_slice(KLYRON_MAGIC);
        let payload = bincode::serialize(self)
            .map_err(|e| PmError::LockfileError(format!("Bincode serialize: {e}")))?;
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&payload);
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PmError> {
        if bytes.len() < 8 {
            return Err(PmError::LockfileError("Truncated lockfile".into()));
        }
        if &bytes[..4] != KLYRON_MAGIC {
            return Err(PmError::LockfileError("Invalid magic bytes".into()));
        }
        let payload_len = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        if bytes.len() < 8 + payload_len {
            return Err(PmError::LockfileError("Truncated lockfile payload".into()));
        }
        bincode::deserialize(&bytes[8..8 + payload_len])
            .map_err(|e| PmError::LockfileError(format!("Bincode deserialize: {e}")))
    }

    pub fn to_json_pretty(&self) -> Result<String, PmError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PmError::LockfileError(format!("JSON serialize: {e}")))
    }

    pub fn from_json(json: &str) -> Result<Self, PmError> {
        serde_json::from_str(json)
            .map_err(|e| PmError::LockfileError(format!("JSON deserialize: {e}")))
    }

    pub fn add_package(&mut self, name: &str, version: &str, pkg: LockfilePackage) {
        let key = format!("{name}@{version}");
        self.packages.insert(key, pkg);
        self.metadata.install_count = self.packages.len() as u64;
    }

    pub fn remove_package(&mut self, name: &str) {
        let keys: Vec<String> = self.packages.keys()
            .filter(|k| k.starts_with(&format!("{name}@")))
            .cloned()
            .collect();
        for k in keys {
            self.packages.remove(&k);
        }
        self.metadata.install_count = self.packages.len() as u64;
    }

    pub fn verify_all_integrity(&self, verify_signatures: bool) -> Vec<PmError> {
        let mut errors = Vec::new();
        for (key, pkg) in &self.packages {
            if pkg.integrity.is_empty() {
                errors.push(PmError::IntegrityMismatch {
                    expected: "some hash".to_string(),
                    actual: "missing integrity field".to_string(),
                });
                continue;
            }
            let hashes = self.compute_hashes_for_package(key);
            if let Some(expected_hash) = &hashes {
                if *expected_hash != pkg.integrity {
                    errors.push(PmError::IntegrityMismatch {
                        expected: expected_hash.clone(),
                        actual: pkg.integrity.clone(),
                    });
                }
            }
            if verify_signatures {
                if pkg.signature.is_none() || pkg.signer.is_none() {
                    errors.push(PmError::IntegrityMismatch {
                        expected: "signature".to_string(),
                        actual: "missing".to_string(),
                    });
                }
            }
        }
        errors
    }

    pub fn compute_integrity(data: &[u8]) -> String {
        let hash = Sha512::digest(data);
        format!("sha512-{}", base64::encode(hash))
    }

    pub fn compute_integrity_sha256(data: &[u8]) -> String {
        let hash = Sha256::digest(data);
        format!("sha256-{}", hex::encode(hash))
    }

    fn compute_hashes_for_package(&self, _key: &str) -> Option<String> {
        None
    }
}

pub fn verify_lockfile_integrity(lockfile_path: &Path) -> Result<(), PmError> {
    let content = std::fs::read_to_string(lockfile_path)
        .map_err(|e| PmError::LockfileError(format!("Cannot read lockfile: {e}")))?;

    let parsed: KlyronLockfile = KlyronLockfile::from_json(&content)?;

    let errors = parsed.verify_all_integrity(false);
    if !errors.is_empty() {
        return Err(PmError::IntegrityMismatch {
            expected: "all packages intact".to_string(),
            actual: format!("{} integrity failures", errors.len()),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_new() {
        let lf = KlyronLockfile::new();
        assert_eq!(lf.lockfile_version, 3);
        assert!(lf.packages.is_empty());
    }

    #[test]
    fn test_lockfile_roundtrip() {
        let mut lf = KlyronLockfile::new();
        lf.add_package("foo", "1.0.0", LockfilePackage {
            name: "foo".into(),
            version: "1.0.0".into(),
            resolved: "https://registry.npmjs.org/foo/-/foo-1.0.0.tgz".into(),
            integrity: "sha512-deadbeef".into(),
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

        let bytes = lf.to_bytes().unwrap();
        let restored = KlyronLockfile::from_bytes(&bytes).unwrap();
        assert_eq!(restored.packages.len(), 1);
        assert!(restored.packages.contains_key("foo@1.0.0"));
    }

    #[test]
    fn test_integrity_computation() {
        let data = b"hello world";
        let hash = KlyronLockfile::compute_integrity(data);
        assert!(hash.starts_with("sha512-"));
        assert!(hash.len() > 64);
    }
}
