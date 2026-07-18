use base64::Engine;

pub struct JSCBuffer {
    data: Vec<u8>,
}

impl JSCBuffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { data: Vec::with_capacity(cap) }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self { data: bytes.to_vec() }
    }

    pub fn from_string(s: &str) -> Self {
        Self { data: s.as_bytes().to_vec() }
    }

    pub fn alloc(size: usize) -> Self {
        Self { data: vec![0u8; size] }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn to_string(&self, encoding: &str) -> Result<String, String> {
        match encoding {
            "utf8" | "utf-8" => String::from_utf8(self.data.clone()).map_err(|e| e.to_string()),
            "hex" => Ok(hex::encode(&self.data)),
            "base64" => Ok(base64::engine::general_purpose::STANDARD.encode(&self.data)),
            "base64url" => Ok(base64::engine::general_purpose::URL_SAFE.encode(&self.data)),
            _ => String::from_utf8(self.data.clone()).map_err(|e| e.to_string()),
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> Self {
        let s = start.min(self.data.len());
        let e = end.min(self.data.len());
        Self { data: self.data[s..e].to_vec() }
    }

    pub fn concat(buffers: &[&JSCBuffer]) -> Self {
        let total: usize = buffers.iter().map(|b| b.len()).sum();
        let mut data = Vec::with_capacity(total);
        for buf in buffers {
            data.extend_from_slice(&buf.data);
        }
        Self { data }
    }

    pub fn copy_from(&mut self, src: &[u8], offset: usize) -> usize {
        let available = self.data.len().saturating_sub(offset);
        let to_copy = src.len().min(available);
        self.data[offset..offset + to_copy].copy_from_slice(&src[..to_copy]);
        to_copy
    }

    pub fn write(&mut self, src: &[u8]) {
        self.data.extend_from_slice(src);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for JSCBuffer {
    fn default() -> Self {
        Self::new()
    }
}
