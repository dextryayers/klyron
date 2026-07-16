use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DebugClient {
    pub id: String,
    pub connected_at: std::time::Instant,
    pub paused: bool,
    pub current_frame_id: Option<String>,
}

impl Serialize for DebugClient {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("DebugClient", 3)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("paused", &self.paused)?;
        s.serialize_field("current_frame_id", &self.current_frame_id)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for DebugClient {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper {
            id: String,
            paused: bool,
            current_frame_id: Option<String>,
        }
        let helper = Helper::deserialize(deserializer)?;
        Ok(Self {
            id: helper.id,
            connected_at: std::time::Instant::now(),
            paused: helper.paused,
            current_frame_id: helper.current_frame_id,
        })
    }
}

impl DebugClient {
    pub fn new(id: String) -> Self {
        Self {
            id,
            connected_at: std::time::Instant::now(),
            paused: false,
            current_frame_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_client_new() {
        let client = DebugClient::new("test-client-1".into());
        assert_eq!(client.id, "test-client-1");
        assert!(!client.paused);
        assert!(client.current_frame_id.is_none());
    }

    #[test]
    fn test_debug_client_serialize() {
        let client = DebugClient::new("client-2".into());
        let json = serde_json::to_string(&client).unwrap();
        assert!(json.contains("client-2"));
        let deserialized: DebugClient = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, client.id);
    }
}
