pub mod binary;
pub mod json;

pub use json::*;

pub fn to_json_string<T: serde::Serialize>(value: &T) -> serde_json::Result<String> {
    json::to_string(value)
}

pub fn from_json_str<T: serde::de::DeserializeOwned>(s: &str) -> serde_json::Result<T> {
    json::from_str(s)
}

pub fn to_binary<T: serde::Serialize>(value: &T) -> binary::Result<Vec<u8>> {
    binary::to_vec(value)
}

pub fn from_binary<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> binary::Result<T> {
    binary::from_slice(bytes)
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
    fn test_json_roundtrip() {
        let obj = TestObj { name: "hello".into(), value: 42 };
        let json = super::to_json_string(&obj).unwrap();
        let back: TestObj = super::from_json_str(&json).unwrap();
        assert_eq!(obj, back);
    }

    #[test]
    fn test_binary_roundtrip() {
        let obj = TestObj { name: "binary".into(), value: 99 };
        let bytes = super::to_binary(&obj).unwrap();
        let back: TestObj = super::from_binary(&bytes).unwrap();
        assert_eq!(obj, back);
    }

    #[test]
    fn test_serde_vec_roundtrip() {
        let data = vec![1, 2, 3, 4, 5];
        let json = super::to_json_string(&data).unwrap();
        let back: Vec<i32> = super::from_json_str(&json).unwrap();
        assert_eq!(data, back);
    }

    #[cfg(feature = "simd")]
    #[test]
    fn test_simd_json() {
        let json = r#"{"name":"test","value":99}"#;
        let obj: TestObj = super::json::simd::from_str(json).unwrap();
        assert_eq!(obj.name, "test");
        assert_eq!(obj.value, 99);
    }
}
