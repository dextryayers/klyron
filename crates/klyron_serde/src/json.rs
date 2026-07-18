use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn to_string<T: Serialize>(value: &T) -> serde_json::Result<String> {
    serde_json::to_string(value)
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

pub fn to_writer<T: Serialize, W: std::io::Write>(value: &T, writer: W) -> serde_json::Result<()> {
    serde_json::to_writer(writer, value)
}

pub fn to_writer_pretty<T: Serialize, W: std::io::Write>(value: &T, writer: W) -> serde_json::Result<()> {
    serde_json::to_writer_pretty(writer, value)
}

pub fn from_reader<T: DeserializeOwned, R: std::io::Read>(reader: R) -> serde_json::Result<T> {
    serde_json::from_reader(reader)
}

pub fn to_value<T: Serialize>(value: &T) -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(value)
}

pub fn from_value<T: DeserializeOwned>(value: serde_json::Value) -> serde_json::Result<T> {
    serde_json::from_value(value)
}

pub fn merge(a: &mut serde_json::Value, b: serde_json::Value) {
    if let (serde_json::Value::Object(a_map), serde_json::Value::Object(b_map)) = (a, b) {
        for (k, v) in b_map {
            if v.is_null() {
                a_map.remove(&k);
            } else if let Some(existing) = a_map.get(&k) {
                if existing.is_object() && v.is_object() {
                    if let Some(dest) = a_map.get_mut(&k) {
                        merge(dest, v);
                    }
                } else {
                    a_map.insert(k, v);
                }
            } else {
                a_map.insert(k, v);
            }
        }
    }
}

pub fn diff(a: &serde_json::Value, b: &serde_json::Value) -> serde_json::Value {
    use serde_json::json;
    match (a, b) {
        (serde_json::Value::Object(a_map), serde_json::Value::Object(b_map)) => {
            let mut result = serde_json::Map::new();
            for (k, v) in b_map {
                if !a_map.contains_key(k) {
                    result.insert(k.clone(), json!({"added": v.clone()}));
                } else if a_map[k] != *v {
                    result.insert(k.clone(), json!({"old": a_map[k], "new": v.clone()}));
                }
            }
            for (k, v) in a_map {
                if !b_map.contains_key(k) {
                    result.insert(k.clone(), json!({"removed": v.clone()}));
                }
            }
            serde_json::Value::Object(result)
        }
        _ if a == b => json!({}),
        _ => json!({"old": a, "new": b}),
    }
}

#[cfg(feature = "simd")]
pub mod simd {
    use serde::de::{DeserializeOwned, Error};
    use serde::Serialize;

    pub fn from_str<T: DeserializeOwned>(s: &str) -> serde_json::Result<T> {
        let mut data = s.as_bytes().to_vec();
        simd_json::from_slice(&mut data)
            .map_err(|e| serde_json::error::Error::custom(format!("simd_json error: {}", e)))
    }

    pub fn to_string<T: Serialize>(value: &T) -> serde_json::Result<String> {
        serde_json::to_string(value)
    }

    pub fn to_vec<T: Serialize>(value: &T) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(value)
    }
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

    #[test]
    fn test_merge_objects() {
        let mut a = serde_json::json!({"a": 1, "b": 2});
        let b = serde_json::json!({"b": 3, "c": 4});
        super::merge(&mut a, b);
        assert_eq!(a["a"], 1);
        assert_eq!(a["b"], 3);
        assert_eq!(a["c"], 4);
    }

    #[test]
    fn test_diff_objects() {
        let a = serde_json::json!({"a": 1, "b": 2});
        let b = serde_json::json!({"a": 1, "b": 3, "c": 4});
        let d = super::diff(&a, &b);
        assert!(d.get("b").is_some());
        assert!(d.get("c").is_some());
        assert!(d.get("a").is_none());
    }

    #[test]
    fn test_to_value() {
        let obj = TestObj { name: "test".into(), value: 99 };
        let v = super::to_value(&obj).unwrap();
        assert_eq!(v["name"], "test");
        assert_eq!(v["value"], 99);
    }

    #[test]
    fn test_from_value() {
        let v = serde_json::json!({"name": "test", "value": 99});
        let obj: TestObj = super::from_value(v).unwrap();
        assert_eq!(obj.name, "test");
        assert_eq!(obj.value, 99);
    }
}
