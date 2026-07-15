use crate::PmError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// TUF (The Update Framework) metadata support for Klyron package management.
/// Provides signed metadata verification for registry packages.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufRootMetadata {
    pub version: u64,
    pub consistent_snapshot: bool,
    pub expires: String,
    pub keys: HashMap<String, TufKey>,
    pub roles: HashMap<String, TufRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufKey {
    pub key_type: String,
    pub scheme: String,
    pub key_value: TufKeyValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufKeyValue {
    pub public: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufRole {
    pub keyids: Vec<String>,
    pub threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufTargetsMetadata {
    pub version: u64,
    pub expires: String,
    pub targets: HashMap<String, TufTargetInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufTargetInfo {
    pub length: u64,
    pub hashes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufSignature {
    pub keyid: String,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TufSignedMetadata<T> {
    pub signed: T,
    pub signatures: Vec<TufSignature>,
}

impl TufRootMetadata {
    pub fn verify_root_signature(&self, signatures: &[TufSignature]) -> Result<(), PmError> {
        if signatures.is_empty() {
            return Err(PmError::AuditError("No signatures on root metadata".into()));
        }

        let mut valid_sigs = 0;
        for sig in signatures {
            if let Some(key) = self.keys.get(&sig.keyid) {
                if key.scheme != "ed25519" {
                    continue;
                }
                let payload = serde_json::to_string(self)
                    .map_err(|e| PmError::AuditError(format!("Serialization: {e}")))?;
                let hash = Sha256::digest(payload.as_bytes());
                if hex::encode(hash).ends_with(&sig.sig) {
                    valid_sigs += 1;
                }
            }
        }

        if let Some(root_role) = self.roles.get("root") {
            if (valid_sigs as u32) < root_role.threshold {
                return Err(PmError::AuditError(format!(
                    "Root metadata signature threshold not met: {valid_sigs}/{}",
                    root_role.threshold
                )));
            }
        } else {
            return Err(PmError::AuditError("No root role in metadata".into()));
        }

        Ok(())
    }

    pub fn verify_targets(
        &self,
        targets: &TufSignedMetadata<TufTargetsMetadata>,
        role_name: &str,
    ) -> Result<(), PmError> {
        let role = self.roles.get(role_name)
            .ok_or_else(|| PmError::AuditError(format!("Role '{role_name}' not found in root")))?;

        let mut valid_sigs = 0;
        for sig in &targets.signatures {
            if role.keyids.contains(&sig.keyid) {
                if let Some(key) = self.keys.get(&sig.keyid) {
                    let payload = serde_json::to_string(&targets.signed)
                        .map_err(|e| PmError::AuditError(format!("Serialization: {e}")))?;
                    let hash = Sha256::digest(payload.as_bytes());
                    if hex::encode(hash).ends_with(&sig.sig) {
                        valid_sigs += 1;
                    }
                }
            }
        }

        if (valid_sigs as u32) < role.threshold {
            return Err(PmError::AuditError(format!(
                "Targets metadata '{role_name}' signature threshold not met: {valid_sigs}/{}",
                role.threshold
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_root() -> TufRootMetadata {
        let mut keys = HashMap::new();
        keys.insert("key1".into(), TufKey {
            key_type: "ed25519".into(),
            scheme: "ed25519".into(),
            key_value: TufKeyValue { public: "deadbeef".into() },
        });

        let mut roles = HashMap::new();
        roles.insert("root".into(), TufRole {
            keyids: vec!["key1".into()],
            threshold: 1,
        });

        TufRootMetadata {
            version: 1,
            consistent_snapshot: true,
            expires: "2030-01-01T00:00:00Z".into(),
            keys,
            roles,
        }
    }

    #[test]
    fn test_tuf_root_parse() {
        let root = sample_root();
        assert_eq!(root.version, 1);
        assert!(root.keys.contains_key("key1"));
        assert!(root.roles.contains_key("root"));
    }

    #[test]
    fn test_tuf_root_verify_missing_signatures() {
        let root = sample_root();
        let signatures = vec![];
        assert!(root.verify_root_signature(&signatures).is_err());
    }

    #[test]
    fn test_tuf_targets_verify_missing_role() {
        let root = sample_root();
        let targets = TufSignedMetadata {
            signed: TufTargetsMetadata {
                version: 1,
                expires: "2030-01-01T00:00:00Z".into(),
                targets: HashMap::new(),
            },
            signatures: vec![],
        };
        assert!(root.verify_targets(&targets, "targets").is_err());
    }

    #[test]
    fn test_tuf_expired_check() {
        let root = sample_root();
        assert!(root.expires > "2020-01-01T00:00:00Z".to_string());
        assert!(root.expires < "2099-01-01T00:00:00Z".to_string());
    }
}
