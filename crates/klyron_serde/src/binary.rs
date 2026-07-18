use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{Read, Write};

#[derive(Debug)]
pub enum BinaryError {
    Serialization(String),
    Deserialization(String),
    Io(std::io::Error),
}

impl std::fmt::Display for BinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryError::Serialization(msg) => write!(f, "serialization error: {msg}"),
            BinaryError::Deserialization(msg) => write!(f, "deserialization error: {msg}"),
            BinaryError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for BinaryError {}

impl From<std::io::Error> for BinaryError {
    fn from(e: std::io::Error) -> Self {
        BinaryError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, BinaryError>;

pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value)
        .map_err(|e| BinaryError::Serialization(e.to_string()))
}

pub fn from_slice<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    bincode::deserialize(bytes)
        .map_err(|e| BinaryError::Deserialization(e.to_string()))
}

pub fn to_writer<T: Serialize, W: Write>(value: &T, writer: &mut W) -> Result<()> {
    let bytes = to_vec(value)?;
    let len = bytes.len() as u64;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(&bytes)?;
    Ok(())
}

pub fn from_reader<T: DeserializeOwned, R: Read>(reader: &mut R) -> Result<T> {
    let mut len_buf = [0u8; 8];
    reader.read_exact(&mut len_buf)?;
    let len = u64::from_le_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    from_slice(&buf)
}

pub fn to_file<T: Serialize>(value: &T, path: &std::path::Path) -> Result<()> {
    let bytes = to_vec(value)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

pub fn from_file<T: DeserializeOwned>(path: &std::path::Path) -> Result<T> {
    let bytes = std::fs::read(path)?;
    from_slice(&bytes)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestData {
        name: String,
        count: u32,
        active: bool,
    }

    #[test]
    fn test_binary_roundtrip() {
        let data = TestData {
            name: "test".into(),
            count: 42,
            active: true,
        };
        let bytes = super::to_vec(&data).unwrap();
        let back: TestData = super::from_slice(&bytes).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn test_binary_writer_reader() {
        let data = TestData {
            name: "writer".into(),
            count: 100,
            active: false,
        };
        let mut buf: Vec<u8> = Vec::new();
        super::to_writer(&data, &mut buf).unwrap();
        let back: TestData = super::from_reader(&mut std::io::Cursor::new(buf)).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn test_binary_file_roundtrip() {
        let data = TestData {
            name: "file_test".into(),
            count: 255,
            active: true,
        };
        let path = std::env::temp_dir().join("klyron_binary_test.bin");
        super::to_file(&data, &path).unwrap();
        let back: TestData = super::from_file(&path).unwrap();
        assert_eq!(data, back);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_binary_empty_struct() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Empty;
        let data = Empty;
        let bytes = super::to_vec(&data).unwrap();
        let back: Empty = super::from_slice(&bytes).unwrap();
        assert_eq!(data, back);
    }

    #[test]
    fn test_binary_ints() {
        let nums: Vec<i64> = vec![-1, 0, 1, i64::MAX, i64::MIN];
        let bytes = super::to_vec(&nums).unwrap();
        let back: Vec<i64> = super::from_slice(&bytes).unwrap();
        assert_eq!(nums, back);
    }
}
