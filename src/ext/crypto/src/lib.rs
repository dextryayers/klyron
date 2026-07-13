use deno_core::{extension, op2, Extension};
use sha2::{Digest, Sha256};

extension!(
  klyron_crypto,
  ops = [op_crypto_random_uuid, op_crypto_random_values, op_crypto_sha256, op_crypto_hex_encode],
  esm_entry_point = "ext:klyron_crypto/crypto.js",
  esm = [dir "js", "crypto.js"],
);

pub fn init() -> Extension {
  klyron_crypto::init()
}

#[op2]
#[string]
fn op_crypto_random_uuid() -> String {
  use std::time::{SystemTime, UNIX_EPOCH};
  let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
  let mut buf = [0u8; 16];
  for (i, b) in buf.iter_mut().enumerate() {
    *b = (ts.as_nanos() >> (i * 8)) as u8 ^ rand_byte();
  }
  buf[6] = (buf[6] & 0x0f) | 0x40;
  buf[8] = (buf[8] & 0x3f) | 0x80;
  format!("{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
    buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
    buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15])
}

fn rand_byte() -> u8 {
  (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos() & 0xff) as u8
}

#[op2]
#[string]
fn op_crypto_random_values(#[string] len_str: String) -> String {
  let len: usize = len_str.parse().unwrap_or(32);
  let mut buf = vec![0u8; len];
  for b in &mut buf {
    *b = rand_byte();
  }
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
