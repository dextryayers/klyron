pub struct JSCArrayBuffer {
    data: Vec<u8>,
}

impl JSCArrayBuffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn with_size(size: usize) -> Self {
        Self { data: vec![0u8; size] }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self { data: bytes.to_vec() }
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

    pub fn slice(&self, start: usize, end: usize) -> Self {
        let s = start.min(self.data.len());
        let e = end.min(self.data.len());
        Self { data: self.data[s..e].to_vec() }
    }

    pub fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0);
    }

    pub fn transfer(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.data)
    }

    pub fn write_at(&mut self, offset: usize, bytes: &[u8]) -> usize {
        let available = self.data.len().saturating_sub(offset);
        let to_copy = bytes.len().min(available);
        self.data[offset..offset + to_copy].copy_from_slice(&bytes[..to_copy]);
        to_copy
    }
}

impl Default for JSCArrayBuffer {
    fn default() -> Self {
        Self::new()
    }
}
