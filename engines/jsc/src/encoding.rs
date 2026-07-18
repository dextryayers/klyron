use base64::Engine;

pub struct JSCEncoding;

impl JSCEncoding {
    pub fn new() -> Self {
        Self
    }

    pub fn encode_utf8(&self, input: &str) -> Vec<u8> {
        input.as_bytes().to_vec()
    }

    pub fn decode_utf8(&self, input: &[u8]) -> Result<String, String> {
        String::from_utf8(input.to_vec()).map_err(|e| e.to_string())
    }

    pub fn encode_base64(&self, input: &[u8]) -> String {
        use base64::engine::general_purpose::STANDARD;
        STANDARD.encode(input)
    }

    pub fn decode_base64(&self, input: &str) -> Result<Vec<u8>, String> {
        use base64::engine::general_purpose::STANDARD;
        STANDARD.decode(input).map_err(|e| e.to_string())
    }

    pub fn encode_hex(&self, input: &[u8]) -> String {
        hex::encode(input)
    }

    pub fn decode_hex(&self, input: &str) -> Result<Vec<u8>, String> {
        hex::decode(input).map_err(|e| e.to_string())
    }

    pub fn encode_base64url(&self, input: &[u8]) -> String {
        use base64::engine::general_purpose::URL_SAFE;
        URL_SAFE.encode(input)
    }

    pub fn decode_base64url(&self, input: &str) -> Result<Vec<u8>, String> {
        use base64::engine::general_purpose::URL_SAFE;
        URL_SAFE.decode(input).map_err(|e| e.to_string())
    }
}

impl Default for JSCEncoding {
    fn default() -> Self {
        Self::new()
    }
}
