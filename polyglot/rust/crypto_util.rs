use crate::types::{KlyronError, Result};
use std::fmt::Write;

pub fn hash(data: &[u8]) -> String {
    sha256(data)
}

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
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    padded
}

pub fn random_bytes(len: usize) -> Result<Vec<u8>> {
    let mut f = std::fs::File::open("/dev/urandom")
        .map_err(|e| KlyronError::Crypto(format!("cannot open /dev/urandom: {}", e)))?;
    use std::io::Read;
    let mut buf = vec![0u8; len];
    f.read_exact(&mut buf)
        .map_err(|e| KlyronError::Crypto(format!("cannot read /dev/urandom: {}", e)))?;
    Ok(buf)
}

pub fn uuid() -> Result<String> {
    let bytes = random_bytes(16)?;
    let mut out = String::with_capacity(36);
    for i in 0..16 {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            out.push('-');
        }
        let b = if i == 6 {
            (bytes[i] & 0x0f) | 0x40
        } else if i == 8 {
            (bytes[i] & 0x3f) | 0x80
        } else {
            bytes[i]
        };
        write!(out, "{:02x}", b).unwrap();
    }
    Ok(out)
}

pub fn hex_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        write!(out, "{:02x}", byte).unwrap();
    }
    out
}

pub fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(KlyronError::Crypto("hex string length must be even".into()));
    }
    let mut out = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i+2], 16)
            .map_err(|_| KlyronError::Crypto("invalid hex character".into()))?;
        out.push(byte);
    }
    Ok(out)
}

pub fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(4 * ((data.len() + 2) / 3));
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let val = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((val >> 18) & 0x3f) as usize] as char);
        out.push(CHARS[((val >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 { CHARS[((val >> 6) & 0x3f) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[(val & 0x3f) as usize] as char } else { '=' });
    }
    out
}

pub fn base64_decode(encoded: &str) -> Result<Vec<u8>> {
    fn idx(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes: Vec<u8> = encoded.bytes().filter(|&c| c != b'=').collect();
    if bytes.len() % 4 == 1 {
        return Err(KlyronError::Crypto("invalid base64 length".into()));
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        if chunk.len() < 2 { break; }
        let a = idx(chunk[0]).ok_or(KlyronError::Crypto("invalid base64 char".into()))?;
        let b = idx(chunk[1]).ok_or(KlyronError::Crypto("invalid base64 char".into()))?;
        let val = ((a as u32) << 18) | ((b as u32) << 12);
        out.push((val >> 16) as u8);
        if let Some(&c) = chunk.get(2) {
            let c = idx(c).ok_or(KlyronError::Crypto("invalid base64 char".into()))?;
            let val = val | ((c as u32) << 6);
            out.push((val >> 8) as u8);
            if let Some(&d) = chunk.get(3) {
                let d = idx(d).ok_or(KlyronError::Crypto("invalid base64 char".into()))?;
                let val = val | d as u32;
                out.push(val as u8);
            }
        }
    }
    Ok(out)
}
