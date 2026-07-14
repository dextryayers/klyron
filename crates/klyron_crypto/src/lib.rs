use sha2::{Sha256, Sha512, Digest};

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
}

pub fn sha256(data: &[u8]) -> String {
    CryptoProvider::new().sha256(data)
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
}
