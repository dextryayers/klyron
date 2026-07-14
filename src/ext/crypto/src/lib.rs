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
