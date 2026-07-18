use rand::RngCore;

pub struct JSCCrypto;

impl JSCCrypto {
    pub fn new() -> Self {
        Self
    }

    pub fn random_bytes(&self, length: usize) -> Vec<u8> {
        let mut buf = vec![0u8; length];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut buf);
        buf
    }

    pub fn random_uuid(&self) -> String {
        let bytes = self.random_bytes(16);
        let mut hex = String::with_capacity(36);
        for (i, &b) in bytes.iter().enumerate() {
            if i == 4 || i == 6 || i == 8 || i == 10 {
                hex.push('-');
            }
            hex.push_str(&format!("{:02x}", b));
        }
        hex
    }

    pub fn random_fill(&self, buf: &mut [u8]) {
        let mut rng = rand::thread_rng();
        rng.fill_bytes(buf);
    }
}

impl Default for JSCCrypto {
    fn default() -> Self {
        Self::new()
    }
}
