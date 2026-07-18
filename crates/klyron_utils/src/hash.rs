use std::hash::Hasher;
use std::path::Path;

use sha2::{Digest, Sha256};

pub struct HashUtil;

impl HashUtil {
    pub fn xxhash64(data: &[u8]) -> u64 {
        let mut hasher = twox_hash::XxHash64::default();
        hasher.write(data);
        hasher.finish()
    }

    pub fn xxhash32(data: &[u8]) -> u32 {
        let mut hasher = twox_hash::XxHash32::default();
        hasher.write(data);
        hasher.finish() as u32
    }

    pub fn sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    pub fn md5(data: &[u8]) -> String {
        let mut hasher = md5::Md5::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    pub fn hash_file(path: &Path) -> anyhow::Result<String> {
        let data = std::fs::read(path)?;
        Ok(Self::sha256(&data))
    }

    pub fn hash_reader<R: std::io::Read>(mut reader: R) -> String {
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buf[..n]),
                Err(_) => break,
            }
        }
        format!("{:x}", hasher.finalize())
    }

    pub fn short_hash(data: &[u8]) -> String {
        let full = Self::sha256(data);
        full[..12].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let h = HashUtil::sha256(b"hello");
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn test_xxhash32() {
        let h = HashUtil::xxhash32(b"hello");
        assert!(h > 0);
    }

    #[test]
    fn test_md5() {
        let h = HashUtil::md5(b"hello");
        assert_eq!(h.len(), 32);
    }

    #[test]
    fn test_short_hash() {
        let h = HashUtil::short_hash(b"hello world");
        assert_eq!(h.len(), 12);
    }

    #[test]
    fn test_hash_reader() {
        let data = b"test data";
        let h = HashUtil::hash_reader(&data[..]);
        assert_eq!(h.len(), 64);
    }
}
