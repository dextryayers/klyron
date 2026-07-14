use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use chacha20poly1305::ChaCha20Poly1305;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2;
use rand::Rng;
use sha2::{Digest, Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};
use uuid::Uuid;
use x25519_dalek::{PublicKey, StaticSecret};

pub struct CryptoProvider;

impl CryptoProvider {
    #[inline]
    pub fn new() -> Self {
        Self
    }

    #[inline]
    pub fn sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    #[inline]
    pub fn sha512(&self, data: &[u8]) -> String {
        let mut hasher = Sha512::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    #[inline]
    pub fn sha3_256(&self, data: &[u8]) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    #[inline]
    pub fn sha3_512(&self, data: &[u8]) -> String {
        let mut hasher = Sha3_512::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    #[inline]
    pub fn sha1(&self, data: &[u8]) -> String {
        let mut hasher = sha1::Sha1::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    #[inline]
    pub fn random_bytes(&self, len: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..len).map(|_| rng.r#gen()).collect()
    }

    #[inline]
    pub fn random_hex(&self, len: usize) -> String {
        hex::encode(self.random_bytes(len))
    }

    #[inline]
    pub fn uuid_v4(&self) -> String {
        Uuid::new_v4().to_string()
    }

    #[inline]
    pub fn uuid_v7(&self) -> String {
        Uuid::now_v7().to_string()
    }

    #[inline]
    pub fn base64_encode(&self, data: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(data)
    }

    #[inline]
    pub fn base64_decode(&self, data: &str) -> anyhow::Result<Vec<u8>> {
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.decode(data)?)
    }

    #[inline]
    pub fn hash_sha256_string(&self, input: &str) -> String {
        self.sha256(input.as_bytes())
    }

    #[inline]
    pub fn hash_sha512_string(&self, input: &str) -> String {
        self.sha512(input.as_bytes())
    }

    #[inline]
    pub fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key).expect("HMAC key size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    #[inline]
    pub fn hmac_sha256_hex(&self, key: &[u8], data: &[u8]) -> String {
        hex::encode(self.hmac_sha256(key, data))
    }

    #[inline]
    pub fn pbkdf2_sha256(&self, password: &[u8], salt: &[u8], rounds: u32, key_len: usize) -> Vec<u8> {
        let mut key = vec![0u8; key_len];
        pbkdf2::<Hmac<Sha256>>(password, salt, rounds, &mut key)
            .expect("PBKDF2 failed");
        key
    }

    #[inline]
    pub fn aes256_gcm_encrypt(&self, key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)?;
        let nonce = Nonce::from_slice(nonce);
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {e}"))?;
        Ok(ciphertext)
    }

    #[inline]
    pub fn aes256_gcm_decrypt(&self, key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)?;
        let nonce = Nonce::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("AES-GCM decryption failed: {e}"))?;
        Ok(plaintext)
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    pub fn chacha20_poly1305_encrypt(&self, key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        use chacha20poly1305::aead::Aead;
        let cipher = ChaCha20Poly1305::new_from_slice(key)?;
        let nonce = chacha20poly1305::Nonce::from_slice(nonce);
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("ChaCha20Poly1305 encryption failed: {e}"))?;
        Ok(ciphertext)
    }

    #[inline]
    pub fn chacha20_poly1305_decrypt(&self, key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        use chacha20poly1305::aead::Aead;
        let cipher = ChaCha20Poly1305::new_from_slice(key)?;
        let nonce = chacha20poly1305::Nonce::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("ChaCha20Poly1305 decryption failed: {e}"))?;
        Ok(plaintext)
    }

    #[inline]
    pub fn ed25519_sign(&self, secret: &[u8; 32], message: &[u8]) -> Vec<u8> {
        let signing_key = SigningKey::from_bytes(secret);
        let signature = signing_key.sign(message);
        signature.to_bytes().to_vec()
    }

    #[inline]
    pub fn ed25519_verify(&self, public: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> anyhow::Result<()> {
        let verifying_key = VerifyingKey::from_bytes(public)
            .map_err(|_| anyhow::anyhow!("Invalid Ed25519 public key"))?;
        let sig = Signature::from_bytes(signature);
        verifying_key.verify(message, &sig)
            .map_err(|e| anyhow::anyhow!("Ed25519 verification failed: {e}"))
    }

    #[inline]
    pub fn ed25519_generate_keypair(&self) -> ([u8; 32], [u8; 32]) {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        (signing_key.to_bytes(), verifying_key.to_bytes())
    }

    #[inline]
    pub fn x25519_key_exchange(&self, private: &[u8; 32], public: &[u8; 32]) -> [u8; 32] {
        let private_key = StaticSecret::from(*private);
        let public_key = PublicKey::from(*public);
        let shared = private_key.diffie_hellman(&public_key);
        shared.to_bytes()
    }

    #[inline]
    pub fn x25519_generate_keypair(&self) -> ([u8; 32], [u8; 32]) {
        let mut rng = rand::thread_rng();
        let mut secret_bytes = [0u8; 32];
        rng.fill(&mut secret_bytes);
        let secret = StaticSecret::from(secret_bytes);
        let public = PublicKey::from(&secret);
        (secret.to_bytes(), public.to_bytes())
    }

    #[inline]
    pub fn generate_aes256_key(&self) -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        key
    }

    #[inline]
    pub fn generate_nonce(&self) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill(&mut nonce);
        nonce
    }
}

impl Default for CryptoProvider {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub fn sha256(data: &[u8]) -> String {
    CryptoProvider::new().sha256(data)
}

#[inline]
pub fn sha512(data: &[u8]) -> String {
    CryptoProvider::new().sha512(data)
}

#[inline]
pub fn sha3_256(data: &[u8]) -> String {
    CryptoProvider::new().sha3_256(data)
}

#[inline]
pub fn sha3_512(data: &[u8]) -> String {
    CryptoProvider::new().sha3_512(data)
}

#[inline]
pub fn uuid_v4() -> String {
    CryptoProvider::new().uuid_v4()
}

#[inline]
pub fn uuid_v7() -> String {
    CryptoProvider::new().uuid_v7()
}

#[inline]
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
    fn test_sha3_256() {
        let hash = sha3_256(b"hello");
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_sha3_512() {
        let hash = sha3_512(b"hello");
        assert_eq!(hash.len(), 128);
    }

    #[test]
    fn test_uuid_v7() {
        let id = uuid_v7();
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn test_chacha20_roundtrip() {
        let crypto = CryptoProvider::new();
        let key = crypto.generate_aes256_key();
        let nonce = crypto.generate_nonce();
        let plaintext = b"Hello, ChaCha20-Poly1305!";
        let ciphertext = crypto.chacha20_poly1305_encrypt(&key, &nonce, plaintext).unwrap();
        let decrypted = crypto.chacha20_poly1305_decrypt(&key, &nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ed25519_sign_verify() {
        let crypto = CryptoProvider::new();
        let (secret, public) = crypto.ed25519_generate_keypair();
        let msg = b"test message";
        let sig = crypto.ed25519_sign(&secret, msg);
        let sig_bytes: &[u8; 64] = sig.as_slice().try_into().unwrap();
        assert!(crypto.ed25519_verify(&public, msg, sig_bytes).is_ok());
    }

    #[test]
    fn test_x25519_key_exchange() {
        let crypto = CryptoProvider::new();
        let (alice_priv, alice_pub) = crypto.x25519_generate_keypair();
        let (bob_priv, bob_pub) = crypto.x25519_generate_keypair();
        let alice_shared = crypto.x25519_key_exchange(&alice_priv, &bob_pub);
        let bob_shared = crypto.x25519_key_exchange(&bob_priv, &alice_pub);
        assert_eq!(alice_shared, bob_shared);
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
}
