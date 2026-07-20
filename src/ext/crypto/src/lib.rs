use deno_core::{extension, op2, Extension};
use deno_error::JsErrorBox;
use hex::FromHex;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use md5::Md5;

type HmacSha256 = Hmac<Sha256>;
type HmacSha384 = Hmac<Sha384>;
type HmacSha512 = Hmac<Sha512>;

extension!(
  klyron_crypto,
  ops = [
    op_crypto_random_uuid, op_crypto_random_values, op_crypto_sha256,
    op_crypto_hex_encode, op_crypto_digest, op_crypto_encrypt, op_crypto_decrypt,
    op_crypto_hmac, op_crypto_pbkdf2, op_crypto_sha512
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

fn op_crypto_random_uuid_impl() -> String {
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
fn op_crypto_random_uuid() -> String {
  op_crypto_random_uuid_impl()
}

fn op_crypto_random_values_impl(len: usize) -> Vec<u8> {
  let mut buf = vec![0u8; len];
  read_entropy(&mut buf);
  buf
}

#[op2]
#[serde]
fn op_crypto_random_values(len: u32) -> Vec<u8> {
  op_crypto_random_values_impl(len as usize)
}

fn op_crypto_sha256_impl(data: Vec<u8>) -> String {
  let hash = Sha256::digest(&data);
  hex::encode(hash)
}

#[op2]
#[string]
fn op_crypto_sha256(#[serde] data: Vec<u8>) -> String {
  op_crypto_sha256_impl(data)
}

fn op_crypto_hex_encode_impl(data: Vec<u8>) -> String {
  hex::encode(&data)
}

#[op2]
#[string]
fn op_crypto_hex_encode(#[serde] data: Vec<u8>) -> String {
  op_crypto_hex_encode_impl(data)
}

fn op_crypto_digest_impl(algorithm: String, data: Vec<u8>) -> Result<String, JsErrorBox> {
  match algorithm.to_uppercase().as_str() {
    "MD5" => {
      let hash = Md5::digest(&data);
      Ok(hex::encode(hash))
    }
    "SHA-1" | "SHA1" => {
      let hash = Sha1::digest(&data);
      Ok(hex::encode(hash))
    }
    "SHA-224" | "SHA224" => {
      let hash = Sha224::digest(&data);
      Ok(hex::encode(hash))
    }
    "SHA-256" | "SHA256" => {
      let hash = Sha256::digest(&data);
      Ok(hex::encode(hash))
    }
    "SHA-384" | "SHA384" => {
      let hash = Sha384::digest(&data);
      Ok(hex::encode(hash))
    }
    "SHA-512" | "SHA512" => {
      let hash = Sha512::digest(&data);
      Ok(hex::encode(hash))
    }
    other => Err(JsErrorBox::generic(format!("Unsupported digest algorithm: {other}")))
  }
}

#[op2]
#[string]
fn op_crypto_digest(#[string] algorithm: String, #[serde] data: Vec<u8>) -> Result<String, JsErrorBox> {
  op_crypto_digest_impl(algorithm, data)
}

fn op_crypto_encrypt_impl(algorithm: String, data: Vec<u8>, key_hex: String, iv_hex: String) -> Result<Vec<u8>, JsErrorBox> {
  match algorithm.to_uppercase().as_str() {
    "AES-256-CBC" => {
      use aes::cipher::{BlockEncrypt, KeyInit};
      let key_bytes = <[u8; 32]>::from_hex(&key_hex).map_err(|e| JsErrorBox::generic(format!("invalid key hex: {e}")))?;
      let iv = <[u8; 16]>::from_hex(&iv_hex).map_err(|e| JsErrorBox::generic(format!("invalid iv hex: {e}")))?;
      let cipher = aes::Aes256::new_from_slice(&key_bytes).map_err(|e| JsErrorBox::generic(format!("invalid key: {e}")))?;
      let mut iv_block = iv;
      let padded = pkcs7_pad(&data, 16);
      let mut ct = padded;
      for chunk in ct.chunks_exact_mut(16) {
        for (c, i) in chunk.iter_mut().zip(iv_block.iter()) {
          *c ^= i;
        }
        cipher.encrypt_block(aes::cipher::generic_array::GenericArray::from_mut_slice(chunk));
        iv_block.copy_from_slice(chunk);
      }
      Ok(ct)
    }
    _ => Err(JsErrorBox::generic(format!("Encrypt algorithm not supported: {algorithm}")))
  }
}

#[op2]
#[serde]
fn op_crypto_encrypt(#[string] algorithm: String, #[serde] data: Vec<u8>, #[string] key_hex: String, #[string] iv_hex: String) -> Result<Vec<u8>, JsErrorBox> {
  op_crypto_encrypt_impl(algorithm, data, key_hex, iv_hex)
}

fn op_crypto_decrypt_impl(algorithm: String, data: Vec<u8>, key_hex: String, iv_hex: String) -> Result<Vec<u8>, JsErrorBox> {
  match algorithm.to_uppercase().as_str() {
    "AES-256-CBC" => {
      use aes::cipher::{BlockDecrypt, KeyInit};
      let key_bytes = <[u8; 32]>::from_hex(&key_hex).map_err(|e| JsErrorBox::generic(format!("invalid key hex: {e}")))?;
      let iv = <[u8; 16]>::from_hex(&iv_hex).map_err(|e| JsErrorBox::generic(format!("invalid iv hex: {e}")))?;
      let cipher = aes::Aes256::new_from_slice(&key_bytes).map_err(|e| JsErrorBox::generic(format!("invalid key: {e}")))?;
      let mut pt = data.clone();
      let block_size = 16usize;
      let mut prev = iv;
      for i in 0..pt.len() / block_size {
        let start = i * block_size;
        let end = start + block_size;
        let current_block = pt[start..end].to_vec();
        let mut block = aes::cipher::generic_array::GenericArray::clone_from_slice(&current_block);
        cipher.decrypt_block(&mut block);
        for j in 0..block_size {
          pt[start + j] = block[j] ^ prev[j];
        }
        prev.copy_from_slice(&current_block);
      }
      // Remove PKCS7 padding
      let pad_len = pt[pt.len() - 1] as usize;
      if pad_len > 0 && pad_len <= 16 {
        pt.truncate(pt.len() - pad_len);
      }
      Ok(pt)
    }
    _ => Err(JsErrorBox::generic(format!("Decrypt algorithm not supported: {algorithm}")))
  }
}

#[op2]
#[serde]
fn op_crypto_decrypt(#[string] algorithm: String, #[serde] data: Vec<u8>, #[string] key_hex: String, #[string] iv_hex: String) -> Result<Vec<u8>, JsErrorBox> {
  op_crypto_decrypt_impl(algorithm, data, key_hex, iv_hex)
}

pub fn pkcs7_unpad(data: &[u8]) -> Result<&[u8], aes::cipher::block_padding::UnpadError> {
  if data.is_empty() { return Err(aes::cipher::block_padding::UnpadError); }
  let pad_len = data[data.len() - 1] as usize;
  if pad_len == 0 || pad_len > data.len() { return Err(aes::cipher::block_padding::UnpadError); }
  if data[data.len() - pad_len..].iter().all(|&b| b == pad_len as u8) {
    Ok(&data[..data.len() - pad_len])
  } else {
    Err(aes::cipher::block_padding::UnpadError)
  }
}

fn op_crypto_hmac_impl(algorithm: String, key_hex: String, data_hex: String) -> Result<String, JsErrorBox> {
  let data = Vec::from_hex(&data_hex).map_err(|e| JsErrorBox::generic(format!("invalid data hex: {e}")))?;
  match algorithm.to_uppercase().as_str() {
    "SHA256" | "SHA-256" => {
      let key = <[u8; 32]>::from_hex(&key_hex).unwrap_or_else(|_| {
        let mut k = vec![0u8; 32];
        let decoded = Vec::from_hex(&key_hex).unwrap_or_default();
        k[..decoded.len().min(32)].copy_from_slice(&decoded[..decoded.len().min(32)]);
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&k);
        arr
      });
      let mut mac = HmacSha256::new_from_slice(&key).map_err(|e| JsErrorBox::generic(format!("HMAC key: {e}")))?;
      mac.update(&data);
      Ok(hex::encode(mac.finalize().into_bytes()))
    }
    "SHA384" | "SHA-384" => {
      let key = <[u8; 48]>::from_hex(&key_hex).unwrap_or_else(|_| {
        let mut k = vec![0u8; 48];
        let decoded = Vec::from_hex(&key_hex).unwrap_or_default();
        k[..decoded.len().min(48)].copy_from_slice(&decoded[..decoded.len().min(48)]);
        let mut arr = [0u8; 48];
        arr.copy_from_slice(&k);
        arr
      });
      let mut mac = HmacSha384::new_from_slice(&key).map_err(|e| JsErrorBox::generic(format!("HMAC key: {e}")))?;
      mac.update(&data);
      Ok(hex::encode(mac.finalize().into_bytes()))
    }
    "SHA512" | "SHA-512" => {
      let key = <[u8; 64]>::from_hex(&key_hex).unwrap_or_else(|_| {
        let mut k = vec![0u8; 64];
        let decoded = Vec::from_hex(&key_hex).unwrap_or_default();
        k[..decoded.len().min(64)].copy_from_slice(&decoded[..decoded.len().min(64)]);
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&k);
        arr
      });
      let mut mac = HmacSha512::new_from_slice(&key).map_err(|e| JsErrorBox::generic(format!("HMAC key: {e}")))?;
      mac.update(&data);
      Ok(hex::encode(mac.finalize().into_bytes()))
    }
    _ => Err(JsErrorBox::generic(format!("HMAC algorithm not supported: {algorithm}")))
  }
}

#[op2]
#[string]
fn op_crypto_hmac(#[string] algorithm: String, #[string] key_hex: String, #[string] data_hex: String) -> Result<String, JsErrorBox> {
  op_crypto_hmac_impl(algorithm, key_hex, data_hex)
}

fn op_crypto_pbkdf2_impl(password: String, salt_hex: String, iterations: u32, keylen: usize) -> Result<String, JsErrorBox> {
  let salt = Vec::from_hex(&salt_hex).map_err(|e| JsErrorBox::generic(format!("invalid salt hex: {e}")))?;
  let mut key = vec![0u8; keylen];
  pbkdf2::pbkdf2::<HmacSha256>(password.as_bytes(), &salt, iterations, &mut key)
    .map_err(|e| JsErrorBox::generic(format!("pbkdf2: {e}")))?;
  Ok(hex::encode(key))
}

#[op2]
#[string]
fn op_crypto_pbkdf2(#[string] password: String, #[string] salt_hex: String, iterations: u32, keylen: u32) -> Result<String, JsErrorBox> {
  op_crypto_pbkdf2_impl(password, salt_hex, iterations, keylen as usize)
}

fn op_crypto_sha512_impl(data: Vec<u8>) -> String {
  let hash = Sha512::digest(&data);
  hex::encode(hash)
}

#[op2]
#[string]
fn op_crypto_sha512(#[serde] data: Vec<u8>) -> String {
  op_crypto_sha512_impl(data)
}


fn pkcs7_pad(data: &[u8], block_size: usize) -> Vec<u8> {
  let pad_len = block_size - (data.len() % block_size);
  let mut padded = data.to_vec();
  padded.extend(std::iter::repeat(pad_len as u8).take(pad_len));
  padded
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
        let uuid = op_crypto_random_uuid_impl();
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().nth(8).unwrap(), '-');
        assert_eq!(uuid.chars().nth(13).unwrap(), '-');
        assert_eq!(uuid.chars().nth(18).unwrap(), '-');
        assert_eq!(uuid.chars().nth(23).unwrap(), '-');
    }

    #[test]
    fn test_random_uuid_version() {
        let uuid = op_crypto_random_uuid_impl();
        let v4_byte_str = &uuid[14..16];
        let v4_byte = u8::from_str_radix(v4_byte_str, 16).unwrap();
        assert_eq!(v4_byte & 0xf0, 0x40);
    }

    #[test]
    fn test_random_uuid_variant() {
        let uuid = op_crypto_random_uuid_impl();
        let variant_byte_str = &uuid[19..21];
        let variant_byte = u8::from_str_radix(variant_byte_str, 16).unwrap();
        assert_eq!(variant_byte & 0xc0, 0x80);
    }

    #[test]
    fn test_random_values_default_length() {
        let values = op_crypto_random_values_impl(32);
        assert_eq!(values.len(), 32);
    }

    #[test]
    fn test_sha256_hash() {
        let hash = op_crypto_sha256_impl(b"hello".to_vec());
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_known() {
        let hash = op_crypto_sha256_impl(b"".to_vec());
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_sha512() {
        let hash = op_crypto_sha512_impl(b"test".to_vec());
        assert_eq!(hash.len(), 128);
    }

    #[test]
    fn test_hex_encode() {
        let encoded = op_crypto_hex_encode_impl(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]);
        assert_eq!(encoded, "48656c6c6f");
    }

    #[test]
    fn test_digest_sha256() {
        let result = op_crypto_digest_impl("SHA-256".to_string(), b"test".to_vec());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 64);
    }

    #[test]
    fn test_digest_sha512() {
        let result = op_crypto_digest_impl("SHA-512".to_string(), b"test".to_vec());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 128);
    }

    #[test]
    fn test_hmac_sha256() {
        let result = op_crypto_hmac_impl(
            "SHA256".to_string(),
            hex::encode(b"key".to_vec()),
            hex::encode(b"The quick brown fox jumps over the lazy dog".to_vec()),
        ).unwrap();
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_pbkdf2() {
        let result = op_crypto_pbkdf2_impl(
            "password".to_string(),
            hex::encode(b"salt".to_vec()),
            100, 32,
        ).unwrap();
        assert_eq!(result.len(), 64);
    }

    #[test]
    fn test_aes_256_cbc_roundtrip() {
        let data = b"hello world this is a test message for aes".to_vec();
        let key = hex::encode(op_crypto_random_values_impl(32));
        let iv = hex::encode(op_crypto_random_values_impl(16));
        let encrypted = op_crypto_encrypt_impl("AES-256-CBC".to_string(), data.clone(), key.clone(), iv.clone()).unwrap();
        let decrypted = op_crypto_decrypt_impl("AES-256-CBC".to_string(), encrypted, key, iv).unwrap();
        assert_eq!(data, decrypted);
    }
}
