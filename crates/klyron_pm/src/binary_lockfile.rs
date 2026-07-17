use crate::{LockfileV3, PmError};
use std::path::Path;

/// Binary lockfile entry
#[derive(Debug, Clone)]
pub struct BinaryEntry {
    pub name: String,
    pub version: String,
    pub resolved_offset: u32,
    pub resolved_len: u32,
    pub integrity_offset: u32,
    pub integrity_len: u32,
    pub dependency_count: u16,
    pub dev_dep_count: u16,
    pub has_dev: bool,
    pub has_optional: bool,
}

/// Klyron-native binary lockfile (mmap-able, zero-copy friendly)
#[derive(Debug, Clone)]
pub struct BinaryLockfile {
    pub magic: [u8; 8],
    pub version: u32,
    pub created_at: u64,
    pub content_hash: [u8; 32],
    pub entries: Vec<BinaryEntry>,
    pub checksum: [u8; 32],
}

impl BinaryLockfile {
    const MAGIC: [u8; 8] = *b"KLYRONLF";

    pub fn from_lockfile(lockfile: &LockfileV3) -> Self {
        let entries: Vec<BinaryEntry> = lockfile.packages.iter().map(|(path, pkg)| {
            BinaryEntry {
                name: path.clone(),
                version: pkg.version.clone(),
                resolved_offset: 0,
                resolved_len: pkg.resolved.as_ref().map(|s| s.len() as u32).unwrap_or(0),
                integrity_offset: 0,
                integrity_len: pkg.integrity.as_ref().map(|s| s.len() as u32).unwrap_or(0),
                dependency_count: pkg.dependencies.as_ref().map(|d| d.len() as u16).unwrap_or(0),
                dev_dep_count: 0,
                has_dev: pkg.dev.unwrap_or(false),
                has_optional: pkg.optional.unwrap_or(false),
            }
        }).collect();

        Self {
            magic: Self::MAGIC,
            version: 1,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            content_hash: [0u8; 32],
            entries,
            checksum: [0u8; 32],
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.magic);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.created_at.to_le_bytes());
        buf.extend_from_slice(&self.content_hash);
        buf.extend_from_slice(&(self.entries.len() as u64).to_le_bytes());
        for entry in &self.entries {
            buf.extend_from_slice(&(entry.name.len() as u16).to_le_bytes());
            buf.extend_from_slice(entry.name.as_bytes());
            buf.extend_from_slice(&(entry.version.len() as u16).to_le_bytes());
            buf.extend_from_slice(entry.version.as_bytes());
            buf.extend_from_slice(&entry.resolved_offset.to_le_bytes());
            buf.extend_from_slice(&entry.resolved_len.to_le_bytes());
            buf.extend_from_slice(&entry.integrity_offset.to_le_bytes());
            buf.extend_from_slice(&entry.integrity_len.to_le_bytes());
            buf.extend_from_slice(&entry.dependency_count.to_le_bytes());
            buf.extend_from_slice(&entry.dev_dep_count.to_le_bytes());
            buf.extend_from_slice(&[entry.has_dev as u8, entry.has_optional as u8]);
        }
        let hash = blake3::hash(&buf);
        let checksum = *hash.as_bytes();
        buf.extend_from_slice(&checksum);
        buf
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), PmError> {
        let bytes = self.to_bytes();
        std::fs::write(path, &bytes)
            .map_err(|e| PmError::IoError(format!("Failed to write binary lockfile: {e}")))
    }

    pub fn estimated_size(&self) -> usize {
        8 + 4 + 8 + 32 + 32 +
        self.entries.iter().map(|e| {
            2 + e.name.len() + 2 + e.version.len() + 4*4 + 2*2 + 2
        }).sum::<usize>()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

pub fn save_as_binary(lockfile: &LockfileV3, path: &Path) -> Result<(), PmError> {
    let binary = BinaryLockfile::from_lockfile(lockfile);
    binary.write_to_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LockfilePackage;
    use std::collections::BTreeMap;

    #[test]
    fn test_binary_lockfile_creation() {
        let mut packages = BTreeMap::new();
        packages.insert("node_modules/foo".to_string(), LockfilePackage {
            version: "1.0.0".to_string(),
            resolved: Some("https://registry.example.com/foo.tgz".to_string()),
            integrity: Some("sha512-deadbeef".to_string()),
            dependencies: None,
            optional_dependencies: None,
            peer_dependencies: None,
            dev: None,
            optional: None,
            bundled: None,
            engines: None,
            os: None,
            cpu: None,
            has_dev_dependencies: None,
        });

        let lockfile = LockfileV3 {
            name: Some("test".to_string()),
            lockfile_version: Some("3".to_string()),
            packages,
            workspaces: None,
            metadata: None,
        };

        let binary = BinaryLockfile::from_lockfile(&lockfile);
        assert_eq!(binary.magic, *b"KLYRONLF");
        assert_eq!(binary.version, 1);
        assert_eq!(binary.entries.len(), 1);
    }

    #[test]
    fn test_binary_lockfile_serialization() {
        let mut packages = BTreeMap::new();
        packages.insert("node_modules/test".to_string(), LockfilePackage {
            version: "2.0.0".to_string(),
            resolved: None,
            integrity: None,
            dependencies: None,
            optional_dependencies: None,
            peer_dependencies: None,
            dev: None,
            optional: None,
            bundled: None,
            engines: None,
            os: None,
            cpu: None,
            has_dev_dependencies: None,
        });

        let lockfile = LockfileV3 {
            name: None,
            lockfile_version: None,
            packages,
            workspaces: None,
            metadata: None,
        };

        let binary = BinaryLockfile::from_lockfile(&lockfile);
        let bytes = binary.to_bytes();

        assert_eq!(&bytes[0..8], b"KLYRONLF");
        assert!(bytes.len() > 32);
        assert!(binary.estimated_size() > 0);
    }

    #[test]
    fn test_binary_empty() {
        let lockfile = LockfileV3 {
            name: None,
            lockfile_version: None,
            packages: BTreeMap::new(),
            workspaces: None,
            metadata: None,
        };
        let binary = BinaryLockfile::from_lockfile(&lockfile);
        assert!(binary.is_empty());
        assert_eq!(binary.len(), 0);
    }
}
