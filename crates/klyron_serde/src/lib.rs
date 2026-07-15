use std::cell::RefCell;

use serde::de::DeserializeOwned;
use serde::Serialize;

thread_local! {
    static SERDE_BUFFER: RefCell<String> = const { RefCell::new(String::with_capacity(4096)) };
}

pub fn to_string<T: Serialize>(value: &T) -> serde_json::Result<String> {
    SERDE_BUFFER.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.clear();
        serde_json::to_writer(std::io::Cursor::new(buf.as_bytes()), value)?;
        Ok(buf.clone())
    })
}

pub fn to_string_pretty<T: Serialize>(value: &T) -> serde_json::Result<String> {
    serde_json::to_string_pretty(value)
}

pub fn from_str<T: DeserializeOwned>(s: &str) -> serde_json::Result<T> {
    serde_json::from_str(s)
}

pub fn to_vec<T: Serialize>(value: &T) -> serde_json::Result<Vec<u8>> {
    serde_json::to_vec(value)
}

pub fn from_slice<T: DeserializeOwned>(v: &[u8]) -> serde_json::Result<T> {
    serde_json::from_slice(v)
}

#[cfg(feature = "simd")]
pub mod simd {
    use serde::de::DeserializeOwned;
    use serde::Serialize;

    pub fn from_str<T: DeserializeOwned>(s: &str) -> serde_json::Result<T> {
        let mut deserializer = simd_json::serde::Deserializer::new(simd_json::BorrowedValue::from(s));
        T::deserialize(&mut deserializer)
    }

    pub fn to_string<T: Serialize>(value: &T) -> serde_json::Result<String> {
        serde_json::to_string(value)
    }

    pub fn to_vec<T: Serialize>(value: &T) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(value)
    }
}

pub fn reuse_buffer(capacity: usize) -> String {
    String::with_capacity(capacity)
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestObj {
        name: String,
        value: u64,
    }

    #[test]
    fn test_serde_roundtrip() {
        let obj = TestObj { name: "hello".into(), value: 42 };
        let json = super::to_string(&obj).unwrap();
        let back: TestObj = super::from_str(&json).unwrap();
        assert_eq!(obj, back);
    }

    #[test]
    fn test_serde_vec_roundtrip() {
        let data = vec![1, 2, 3, 4, 5];
        let json = super::to_vec(&data).unwrap();
        let back: Vec<i32> = super::from_slice(&json).unwrap();
        assert_eq!(data, back);
    }

    #[cfg(feature = "simd")]
    #[test]
    fn test_simd_json() {
        let json = r#"{"name":"test","value":99}"#;
        let obj: TestObj = super::simd::from_str(json).unwrap();
        assert_eq!(obj.name, "test");
        assert_eq!(obj.value, 99);
    }
}
