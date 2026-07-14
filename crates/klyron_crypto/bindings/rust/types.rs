pub struct CryptoProvider;
impl CryptoProvider {
    pub fn new() -> Self { Self }
    pub fn sha256(&self, data: &[u8]) -> String { let mut h = sha2::Sha256::new(); h.update(data); hex::encode(h.finalize()) }
    pub fn sha512(&self, data: &[u8]) -> String { let mut h = sha2::Sha512::new(); h.update(data); hex::encode(h.finalize()) }
    pub fn sha1(&self, data: &[u8]) -> String { let mut h = sha1::Sha1::new(); h.update(data); hex::encode(h.finalize()) }
    pub fn random_bytes(&self, len: usize) -> Vec<u8> { use rand::Rng; (0..len).map(|_| rand::thread_rng().r#gen()).collect() }
    pub fn random_hex(&self, len: usize) -> String { hex::encode(self.random_bytes(len)) }
    pub fn uuid_v4(&self) -> String { uuid::Uuid::new_v4().to_string() }
    pub fn base64_encode(&self, data: &[u8]) -> String { use base64::Engine; base64::engine::general_purpose::STANDARD.encode(data) }
    pub fn base64_decode(&self, data: &str) -> anyhow::Result<Vec<u8>> { use base64::Engine; Ok(base64::engine::general_purpose::STANDARD.decode(data)?) }
    pub fn hash_sha256_string(&self, input: &str) -> String { self.sha256(input.as_bytes()) }
    pub fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Vec<u8> { let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(key).expect("HMAC"); mac.update(data); mac.finalize().into_bytes().to_vec() }
    pub fn hmac_sha256_hex(&self, key: &[u8], data: &[u8]) -> String { hex::encode(self.hmac_sha256(key, data)) }
    pub fn pbkdf2_sha256(&self, password: &[u8], salt: &[u8], rounds: u32, key_len: usize) -> Vec<u8> { let mut k = vec![0u8; key_len]; pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(password, salt, rounds, &mut k).expect("PBKDF2"); k }
    pub fn aes256_gcm_encrypt(&self, key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8]) -> anyhow::Result<Vec<u8>> { use aes_gcm::aead::{Aead, KeyInit}; let cipher = aes_gcm::Aes256Gcm::new_from_slice(key)?; let n = aes_gcm::Nonce::from_slice(nonce); cipher.encrypt(n, plaintext).map_err(|e| anyhow::anyhow!("encrypt failed: {e}")) }
    pub fn aes256_gcm_decrypt(&self, key: &[u8; 32], nonce: &[u8; 12], ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> { use aes_gcm::aead::{Aead, KeyInit}; let cipher = aes_gcm::Aes256Gcm::new_from_slice(key)?; let n = aes_gcm::Nonce::from_slice(nonce); cipher.decrypt(n, ciphertext).map_err(|e| anyhow::anyhow!("decrypt failed: {e}")) }
    pub fn generate_aes256_key(&self) -> [u8; 32] { let mut k = [0u8; 32]; rand::Rng::fill(&mut rand::thread_rng(), &mut k); k }
    pub fn generate_nonce(&self) -> [u8; 12] { let mut n = [0u8; 12]; rand::Rng::fill(&mut rand::thread_rng(), &mut n); n }
}
impl Default for CryptoProvider { fn default() -> Self { Self::new() } }
