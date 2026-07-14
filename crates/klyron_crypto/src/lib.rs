use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

pub struct CryptoProvider;

impl CryptoProvider {
    pub fn new() -> Self { Self }

    pub fn sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    pub fn sha512(&self, data: &[u8]) -> String {
        let mut hasher = Sha512::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    pub fn sha1(&self, data: &[u8]) -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    pub fn random_bytes(&self, len: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        use rand::Rng;
        (0..len).map(|_| Rng::r#gen(&mut rng)).collect()
    }

    pub fn random_hex(&self, len: usize) -> String {
        hex::encode(self.random_bytes(len))
    }

    pub fn uuid_v4(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    pub fn base64_encode(&self, data: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(data)
    }

    pub fn base64_decode(&self, data: &str) -> anyhow::Result<Vec<u8>> {
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.decode(data)?)
    }

    pub fn hash_sha256_string(&self, input: &str) -> String {
        self.sha256(input.as_bytes())
    }

    pub fn hash_sha512_string(&self, input: &str) -> String {
        self.sha512(input.as_bytes())
    }

    pub fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key).expect("HMAC key size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    pub fn hmac_sha256_hex(&self, key: &[u8], data: &[u8]) -> String {
        hex::encode(self.hmac_sha256(key, data))
    }

    pub fn pbkdf2_sha256(&self, password: &[u8], salt: &[u8], rounds: u32, key_len: usize) -> Vec<u8> {
        let mut key = vec![0u8; key_len];
        pbkdf2::pbkdf2::<Hmac<Sha256>>(password, salt, rounds, &mut key)
            .expect("PBKDF2 failed");
        key
    }

    pub fn aes256_gcm_encrypt(&self, key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)?;
        let nonce = Nonce::from_slice(nonce);
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {e}"))?;
        Ok(ciphertext)
    }

    pub fn aes256_gcm_decrypt(&self, key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)?;
        let nonce = Nonce::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("AES-GCM decryption failed: {e}"))?;
        Ok(plaintext)
    }

    pub fn aes256_gcm_encrypt_base64(&self, key_b64: &str, nonce_b64: &str, plaintext: &str) -> anyhow::Result<String> {
        use base64::Engine;
        let key_bytes = base64::engine::general_purpose::STANDARD.decode(key_b64)?;
        let nonce_bytes = base64::engine::general_purpose::STANDARD.decode(nonce_b64)?;
        let key: &[u8; 32] = key_bytes.as_slice().try_into()
            .map_err(|_| anyhow::anyhow!("Key must be 32 bytes"))?;
        let nonce: &[u8; 12] = nonce_bytes.as_slice().try_into()
            .map_err(|_| anyhow::anyhow!("Nonce must be 12 bytes"))?;
        let ciphertext = self.aes256_gcm_encrypt(key, nonce, plaintext.as_bytes())?;
        Ok(base64::engine::general_purpose::STANDARD.encode(ciphertext))
    }

    pub fn aes256_gcm_decrypt_base64(&self, key_b64: &str, nonce_b64: &str, ciphertext_b64: &str) -> anyhow::Result<String> {
        use base64::Engine;
        let key_bytes = base64::engine::general_purpose::STANDARD.decode(key_b64)?;
        let nonce_bytes = base64::engine::general_purpose::STANDARD.decode(nonce_b64)?;
        let ciphertext = base64::engine::general_purpose::STANDARD.decode(ciphertext_b64)?;
        let key: &[u8; 32] = key_bytes.as_slice().try_into()
            .map_err(|_| anyhow::anyhow!("Key must be 32 bytes"))?;
        let nonce: &[u8; 12] = nonce_bytes.as_slice().try_into()
            .map_err(|_| anyhow::anyhow!("Nonce must be 12 bytes"))?;
        let plaintext = self.aes256_gcm_decrypt(key, nonce, &ciphertext)?;
        Ok(String::from_utf8(plaintext)?)
    }

    pub fn generate_aes256_key(&self) -> [u8; 32] {
        let mut key = [0u8; 32];
        use rand::Rng;
        rand::thread_rng().fill(&mut key);
        key
    }

    pub fn generate_nonce(&self) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        use rand::Rng;
        rand::thread_rng().fill(&mut nonce);
        nonce
    }
}

impl Default for CryptoProvider {
    fn default() -> Self { Self::new() }
}

pub fn sha256(data: &[u8]) -> String {
    CryptoProvider::new().sha256(data)
}

pub fn sha512(data: &[u8]) -> String {
    CryptoProvider::new().sha512(data)
}

pub fn uuid_v4() -> String {
    CryptoProvider::new().uuid_v4()
}

pub fn random_bytes(len: usize) -> Vec<u8> {
    CryptoProvider::new().random_bytes(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256(b"hello");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_sha512() {
        let hash = sha512(b"hello");
        assert_eq!(hash.len(), 128);
    }

    #[test]
    fn test_sha1() {
        let crypto = CryptoProvider::new();
        let hash = crypto.sha1(b"hello");
        assert_eq!(hash.len(), 40);
        assert_eq!(hash, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }

    #[test]
    fn test_uuid() {
        let id = uuid_v4();
        assert_eq!(id.len(), 36);
        assert_eq!(id.chars().filter(|&c| c == '-').count(), 4);
    }

    #[test]
    fn test_random_bytes() {
        let bytes = random_bytes(16);
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_base64_roundtrip() {
        let crypto = CryptoProvider::new();
        let encoded = crypto.base64_encode(b"hello");
        let decoded = crypto.base64_decode(&encoded).unwrap();
        assert_eq!(decoded, b"hello");
    }

    #[test]
    fn test_hmac_sha256() {
        let crypto = CryptoProvider::new();
        let hmac = crypto.hmac_sha256_hex(b"key", b"data");
        assert_eq!(hmac.len(), 64);
    }

    #[test]
    fn test_pbkdf2() {
        let crypto = CryptoProvider::new();
        let key = crypto.pbkdf2_sha256(b"password", b"salt", 100, 32);
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_aes256_gcm_roundtrip() {
        let crypto = CryptoProvider::new();
        let key = crypto.generate_aes256_key();
        let nonce = crypto.generate_nonce();
        let plaintext = b"Hello, AES-256-GCM!";

        let ciphertext = crypto.aes256_gcm_encrypt(&key, &nonce, plaintext).unwrap();
        let decrypted = crypto.aes256_gcm_decrypt(&key, &nonce, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes256_gcm_base64() {
        let crypto = CryptoProvider::new();
        let key = crypto.generate_aes256_key();
        let nonce = crypto.generate_nonce();

        use base64::Engine;
        let key_b64 = base64::engine::general_purpose::STANDARD.encode(key);
        let nonce_b64 = base64::engine::general_purpose::STANDARD.encode(nonce);

        let encrypted = crypto.aes256_gcm_encrypt_base64(&key_b64, &nonce_b64, "secret data").unwrap();
        let decrypted = crypto.aes256_gcm_decrypt_base64(&key_b64, &nonce_b64, &encrypted).unwrap();

        assert_eq!(decrypted, "secret data");
    }
}
