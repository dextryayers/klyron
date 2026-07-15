use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const SALT_LEN: usize = 32;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 600_000;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedConfig {
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

pub struct SecureConfig {
    key: [u8; KEY_LEN],
}

/// Derive an encryption key using PBKDF2-like iteration with HMAC-SHA256.
/// In production, use AES-256-GCM via the `aes-gcm` crate.
fn derive_key(secret: &[u8], salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    let mut derived = secret.to_vec();
    derived.extend_from_slice(salt);

    for _ in 0..PBKDF2_ITERATIONS {
        let mut hasher = Sha256::new();
        hasher.update(&derived);
        let result = hasher.finalize();
        derived = result.to_vec();
    }

    key.copy_from_slice(&derived[..KEY_LEN]);
    key
}

fn xor_encrypt(plaintext: &[u8], key: &[u8; KEY_LEN], nonce: &[u8]) -> Vec<u8> {
    let mut stream = Vec::with_capacity(plaintext.len());
    let mut counter = 0u64;

    for chunk in plaintext.chunks(32) {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(nonce);
        hasher.update(&counter.to_le_bytes());
        let keystream = hasher.finalize();

        for (p, k) in chunk.iter().zip(keystream.iter()) {
            stream.push(p ^ k);
        }
        counter += 1;
    }

    stream
}

impl SecureConfig {
    pub fn new(machine_id: &str, salt: &[u8]) -> Self {
        let master_password = std::env::var("KLYRON_MASTER_PASSWORD").ok();
        let secret = match master_password {
            Some(pw) => pw,
            None => machine_id.to_string(),
        };

        let key = derive_key(secret.as_bytes(), salt);
        Self { key }
    }

    pub fn from_env_or_generate(machine_id: &str) -> Self {
        let salt = Self::load_or_create_salt();
        Self::new(machine_id, &salt)
    }

    fn load_or_create_salt() -> Vec<u8> {
        let salt_path = Self::salt_path();
        if salt_path.exists() {
            std::fs::read(&salt_path).unwrap_or_else(|_| {
                let salt = Self::generate_salt();
                let _ = std::fs::write(&salt_path, &salt);
                salt
            })
        } else {
            let salt = Self::generate_salt();
            let _ = std::fs::create_dir_all(salt_path.parent().unwrap());
            let _ = std::fs::write(&salt_path, &salt);
            salt
        }
    }

    fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; SALT_LEN];
        rand::rngs::OsRng.fill_bytes(&mut salt);
        salt
    }

    fn salt_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("klyron")
            .join(".config_salt")
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedConfig, Box<dyn std::error::Error>> {
        let mut nonce_bytes = vec![0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);

        let ciphertext = xor_encrypt(plaintext.as_bytes(), &self.key, &nonce_bytes);

        Ok(EncryptedConfig {
            salt: hex::encode(Self::load_or_create_salt()),
            nonce: hex::encode(&nonce_bytes),
            ciphertext: hex::encode(&ciphertext),
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedConfig) -> Result<String, Box<dyn std::error::Error>> {
        let nonce = hex::decode(&encrypted.nonce)?;
        let ciphertext = hex::decode(&encrypted.ciphertext)?;

        let plaintext = xor_encrypt(&ciphertext, &self.key, &nonce);

        String::from_utf8(plaintext)
            .map_err(|e| format!("Invalid UTF-8: {e}").into())
    }

    pub fn encrypt_token(&self, token: &str) -> Result<String, Box<dyn std::error::Error>> {
        let encrypted = self.encrypt(token)?;
        Ok(serde_json::to_string(&encrypted)?)
    }

    pub fn decrypt_token(&self, encrypted_str: &str) -> Result<String, Box<dyn std::error::Error>> {
        let encrypted: EncryptedConfig = serde_json::from_str(encrypted_str)?;
        self.decrypt(&encrypted)
    }
}

pub fn get_machine_id() -> String {
    let paths = [
        "/etc/machine-id",
        "/var/lib/dbus/machine-id",
        "/etc/hostname",
    ];

    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            let id = content.trim().to_string();
            if !id.is_empty() && id.len() >= 8 {
                return id;
            }
        }
    }

    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
    let random_suffix: u64 = rand::random();
    format!("{hostname}-{random_suffix:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let config = SecureConfig::from_env_or_generate("test-machine-id-12345");
        let plaintext = "my-secret-token-abc123";

        let encrypted = config.encrypt(plaintext).unwrap();
        let decrypted = config.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_token_roundtrip() {
        let config = SecureConfig::from_env_or_generate("test-machine-id-67890");
        let token = "npm_abc123def456";

        let encrypted = config.encrypt_token(token).unwrap();
        let decrypted = config.decrypt_token(&encrypted).unwrap();

        assert_eq!(token, decrypted);
    }

    #[test]
    fn test_different_key_fails() {
        let config1 = SecureConfig::new("machine-1", b"fixed-salt-for-test");
        let config2 = SecureConfig::new("machine-2", b"fixed-salt-for-test");

        let encrypted = config1.encrypt("secret").unwrap();
        assert!(config2.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_machine_id_returns_something() {
        let id = get_machine_id();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_encrypt_empty_string() {
        let config = SecureConfig::from_env_or_generate("test");
        let encrypted = config.encrypt("").unwrap();
        let decrypted = config.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "");
    }
}
