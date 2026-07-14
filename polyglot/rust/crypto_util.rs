//! Cryptographic utilities using only the standard library.
//!
//! Provides SHA-256 hashing, cryptographically-secure random
//! byte generation (via `/dev/urandom`), and UUID v4 generation.

use crate::types::{KlyronError, Result};
use std::fmt::Write;

/// Compute the SHA-256 hash of `data` and return it as a lowercase hex string.
pub fn hash(data: &[u8]) -> String {
    sha256(data)
}

// ---------------------------------------------------------------------------
// SHA-256 implementation (FIPS 180-4)
// ---------------------------------------------------------------------------

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
    0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
    0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(data: &[u8]) -> String {
    let msg = pad_message(data);
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for t in 0..16 {
            w[t] = u32::from_be_bytes([
                chunk[t * 4],
                chunk[t * 4 + 1],
                chunk[t * 4 + 2],
                chunk[t * 4 + 3],
            ]);
        }
        for t in 16..64 {
            let s0 = w[t - 15].rotate_right(7)
                ^ w[t - 15].rotate_right(18)
                ^ (w[t - 15] >> 3);
            let s1 = w[t - 2].rotate_right(17)
                ^ w[t - 2].rotate_right(19)
                ^ (w[t - 2] >> 10);
            w[t] = w[t - 16]
                .wrapping_add(s0)
                .wrapping_add(w[t - 7])
                .wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d) = (h[0], h[1], h[2], h[3]);
        let (mut e, mut f, mut g, mut hh) = (h[4], h[5], h[6], h[7]);

        for t in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[t])
                .wrapping_add(w[t]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = String::with_capacity(64);
    for word in &h {
        write!(out, "{:08x}", word).unwrap();
    }
    out
}

fn pad_message(data: &[u8]) -> Vec<u8> {
    let bit_len = (data.len() as u64) * 8;
    let mut padded = data.to_vec();

    // Append 0x80
    padded.push(0x80);

    // Pad with zeros until (len % 64) == 56
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }

    // Append the bit length as a 64-bit big-endian integer
    padded.extend_from_slice(&bit_len.to_be_bytes());
    padded
}

// ---------------------------------------------------------------------------
// Random bytes & UUID
// ---------------------------------------------------------------------------

/// Fill a buffer with cryptographically-secure random bytes
/// read from `/dev/urandom`.
///
/// On non-Unix platforms this returns an error.
pub fn random_bytes(len: usize) -> Result<Vec<u8>> {
    let mut f = std::fs::File::open("/dev/urandom")
        .map_err(|e| KlyronError::Crypto(format!("cannot open /dev/urandom: {}", e)))?;
    use std::io::Read;
    let mut buf = vec![0u8; len];
    f.read_exact(&mut buf)
        .map_err(|e| KlyronError::Crypto(format!("cannot read /dev/urandom: {}", e)))?;
    Ok(buf)
}

/// Generate a random UUID v4 string.
///
/// Format: `xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`
/// where `x` is random and `y` encodes the variant (10xx).
pub fn uuid() -> Result<String> {
    let bytes = random_bytes(16)?;
    let mut out = String::with_capacity(36);
    for i in 0..16 {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            out.push('-');
        }
        let b = if i == 6 {
            // version 4
            (bytes[i] & 0x0f) | 0x40
        } else if i == 8 {
            // variant 10xx
            (bytes[i] & 0x3f) | 0x80
        } else {
            bytes[i]
        };
        write!(out, "{:02x}", b).unwrap();
    }
    Ok(out)
}
