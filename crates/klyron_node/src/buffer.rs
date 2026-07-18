use crate::NodeError;

#[derive(Debug, Clone)]
pub struct Buffer(Vec<u8>);

impl Buffer {
    pub fn alloc(size: usize) -> Self {
        Self(vec![0u8; size])
    }

    pub fn alloc_filled(size: usize, fill: u8) -> Self {
        Self(vec![fill; size])
    }

    pub fn from<T: AsRef<[u8]>>(data: T) -> Self {
        Self(data.as_ref().to_vec())
    }

    pub fn concat(buffers: &[Buffer]) -> Self {
        let cap = buffers.iter().map(|b| b.len()).sum();
        let mut out = Vec::with_capacity(cap);
        for b in buffers {
            out.extend_from_slice(&b.0);
        }
        Self(out)
    }

    pub fn byte_length<T: AsRef<[u8]>>(data: T) -> usize {
        data.as_ref().len()
    }

    pub fn to_string(&self, encoding: &str) -> Result<String, NodeError> {
        match encoding {
            "utf8" | "utf-8" => Ok(String::from_utf8_lossy(&self.0).into_owned()),
            "hex" => Ok(hex::encode(&self.0)),
            "base64" => Ok(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &self.0,
            )),
            "ascii" => Ok(self.0.iter().map(|&b| b as char).collect()),
            _ => Err(NodeError::TypeError(format!("Unknown encoding: {encoding}"))),
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn slice(&self, start: usize, end: usize) -> Result<Self, NodeError> {
        if start > end || end > self.0.len() {
            return Err(NodeError::RangeError("Buffer slice out of bounds".into()));
        }
        Ok(Self(self.0[start..end].to_vec()))
    }

    pub fn write(&mut self, data: &[u8], offset: usize) -> Result<usize, NodeError> {
        if offset + data.len() > self.0.len() {
            return Err(NodeError::RangeError("Buffer write out of bounds".into()));
        }
        self.0[offset..offset + data.len()].copy_from_slice(data);
        Ok(data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_alloc() {
        let b = Buffer::alloc(10);
        assert_eq!(b.len(), 10);
        assert_eq!(b.as_slice(), &[0u8; 10]);
    }

    #[test]
    fn test_buffer_from() {
        let b = Buffer::from("hello");
        assert_eq!(b.to_string("utf8").unwrap(), "hello");
        assert_eq!(b.to_string("hex").unwrap(), "68656c6c6f");
        assert!(b.to_string("base64").is_ok());
    }

    #[test]
    fn test_buffer_concat() {
        let a = Buffer::from("ab");
        let b = Buffer::from("cd");
        let c = Buffer::concat(&[a, b]);
        assert_eq!(c.to_string("utf8").unwrap(), "abcd");
    }

    #[test]
    fn test_buffer_byte_length() {
        assert_eq!(Buffer::byte_length("hello"), 5);
        assert_eq!(Buffer::byte_length(b"hi"), 2);
    }

    #[test]
    fn test_buffer_slice() {
        let b = Buffer::from("hello");
        let s = b.slice(1, 4).unwrap();
        assert_eq!(s.to_string("utf8").unwrap(), "ell");
    }

    #[test]
    fn test_buffer_slice_out_of_bounds() {
        let b = Buffer::from("hi");
        assert!(b.slice(0, 10).is_err());
    }

    #[test]
    fn test_buffer_write() {
        let mut b = Buffer::alloc(10);
        let n = b.write(b"abc", 2).unwrap();
        assert_eq!(n, 3);
        assert_eq!(b.to_string("utf8").unwrap(), "\0\0abc\0\0\0\0\0");
    }

    #[test]
    fn test_buffer_empty() {
        let b = Buffer::alloc(0);
        assert!(b.is_empty());
        assert_eq!(b.len(), 0);
    }
}
