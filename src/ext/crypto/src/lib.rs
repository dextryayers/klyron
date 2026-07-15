use deno_core::{extension, op2, Extension};
use deno_error::JsErrorBox;
use sha2::{Digest, Sha256};

extension!(
  klyron_crypto,
  ops = [
    op_crypto_random_uuid, op_crypto_random_values, op_crypto_sha256,
    op_crypto_hex_encode, op_crypto_digest, op_crypto_encrypt, op_crypto_decrypt
  ],
  esm_entry_point = "ext:klyron_crypto/crypto.js",
  esm = [dir "js", "crypto.js"],
);

pub fn init() -> Extension {
  klyron_crypto::init()
}

fn read_entropy(buf: &mut [u8]) {
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = std::io::Read::read_exact(&mut f, buf);
    } else {
        for b in buf.iter_mut() {
            *b = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                ^ 0xdead_beef_dead_beef_u128) as u8;
        }
    }
}

#[op2]
#[string]
fn op_crypto_random_uuid() -> String {
  let mut buf = [0u8; 16];
  read_entropy(&mut buf);
  buf[6] = (buf[6] & 0x0f) | 0x40;
  buf[8] = (buf[8] & 0x3f) | 0x80;
  format!("{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
    buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
    buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15])
}

#[op2]
#[string]
fn op_crypto_random_values(#[string] len_str: String) -> String {
  let len: usize = len_str.parse().unwrap_or(32);
  let mut buf = vec![0u8; len];
  read_entropy(&mut buf);
  hex::encode(buf)
}

#[op2]
#[string]
fn op_crypto_sha256(#[string] data: String) -> String {
  let hash = Sha256::digest(data.as_bytes());
  hex::encode(hash)
}

#[op2]
#[string]
fn op_crypto_hex_encode(#[serde] data: Vec<u8>) -> String {
  hex::encode(&data)
}

#[op2]
#[string]
fn op_crypto_digest(#[string] algorithm: String, #[serde] data: Vec<u8>) -> Result<String, JsErrorBox> {
  match algorithm.to_uppercase().as_str() {
    "SHA-256" | "SHA256" => {
      let hash = Sha256::digest(&data);
      Ok(hex::encode(hash))
    }
    other => Err(JsErrorBox::generic(format!("Unsupported digest algorithm: {other}")))
  }
}

#[op2]
#[string]
fn op_crypto_encrypt(#[string] _algorithm: String, #[serde] _data: Vec<u8>, #[string] _key: String) -> Result<String, JsErrorBox> {
  Err(JsErrorBox::generic("SubtleCrypto.encrypt not yet implemented"))
}

#[op2]
#[string]
fn op_crypto_decrypt(#[string] _algorithm: String, #[serde] _data: Vec<u8>, #[string] _key: String) -> Result<String, JsErrorBox> {
  Err(JsErrorBox::generic("SubtleCrypto.decrypt not yet implemented"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_returns_extension() {
        let ext = init();
        assert_eq!(ext.name, "klyron_crypto");
    }

    #[test]
    fn test_random_uuid_format() {
        let uuid = op_crypto_random_uuid();
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().nth(8).unwrap(), '-');
        assert_eq!(uuid.chars().nth(13).unwrap(), '-');
        assert_eq!(uuid.chars().nth(18).unwrap(), '-');
        assert_eq!(uuid.chars().nth(23).unwrap(), '-');
    }

    #[test]
    fn test_random_uuid_version() {
        let uuid = op_crypto_random_uuid();
        let v4_byte_str = &uuid[14..16];
        let v4_byte = u8::from_str_radix(v4_byte_str, 16).unwrap();
        assert_eq!(v4_byte & 0xf0, 0x40);
    }

    #[test]
    fn test_random_uuid_variant() {
        let uuid = op_crypto_random_uuid();
        let variant_byte_str = &uuid[19..21];
        let variant_byte = u8::from_str_radix(variant_byte_str, 16).unwrap();
        assert_eq!(variant_byte & 0xc0, 0x80);
    }

    #[test]
    fn test_random_values_default_length() {
        let values = op_crypto_random_values("32".to_string());
        assert_eq!(values.len(), 64);
    }

    #[test]
    fn test_random_values_variable() {
        let values = op_crypto_random_values("16".to_string());
        assert_eq!(values.len(), 32);
    }

    #[test]
    fn test_random_values_zero_length() {
        let values = op_crypto_random_values("0".to_string());
        assert_eq!(values, "");
    }

    #[test]
    fn test_sha256_hash() {
        let hash = op_crypto_sha256("hello".to_string());
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_known() {
        let hash = op_crypto_sha256("".to_string());
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_hex_encode() {
        let encoded = op_crypto_hex_encode(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]);
        assert_eq!(encoded, "48656c6c6f");
    }

    #[test]
    fn test_hex_encode_empty() {
        let encoded = op_crypto_hex_encode(vec![]);
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_digest_sha256() {
        let result = op_crypto_digest("SHA-256".to_string(), b"test".to_vec());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 64);
    }

    #[test]
    fn test_digest_unsupported() {
        let result = op_crypto_digest("MD5".to_string(), b"test".to_vec());
        assert!(result.is_err());
    }

    #[test]
    fn test_digest_sha256_short() {
        let result = op_crypto_digest("SHA256".to_string(), b"data".to_vec());
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_not_implemented() {
        let result = op_crypto_encrypt("AES-GCM".to_string(), vec![1, 2, 3], "key".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_not_implemented() {
        let result = op_crypto_decrypt("AES-GCM".to_string(), vec![1, 2, 3], "key".to_string());
        assert!(result.is_err());
    }
}
