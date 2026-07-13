import { op_crypto_random_uuid, op_crypto_random_values, op_crypto_sha256, op_crypto_hex_encode } from "ext:core/ops";

export function randomBytes(size) {
  const buf = new Uint8Array(size);
  const bytes = op_crypto_random_values(size);
  for (let i = 0; i < size; i++) buf[i] = bytes[i];
  return buf;
}

export function randomUUID() { return op_crypto_random_uuid(); }

export function randomFillSync(buf, offset, size) {
  offset = offset || 0;
  size = size || buf.length - offset;
  const bytes = op_crypto_random_values(size);
  for (let i = 0; i < size; i++) buf[offset + i] = bytes[i];
  return buf;
}

class Hash {
  constructor() { this._data = ""; }
  update(data, enc) { this._data += typeof data === "string" ? data : new TextDecoder().decode(data); return this; }
  digest(enc) {
    const hex = op_crypto_sha256(this._data);
    if (enc === "hex") return hex;
    if (enc === "base64") return btoa?.(hex.match(/../g).map(b => String.fromCharCode(parseInt(b, 16))).join("")) || hex;
    return Buffer.from(hex, "hex");
  }
}

export function createHash(algorithm) {
  if (algorithm?.toLowerCase() === "sha256") return new Hash();
  throw new Error(`Hash algorithm not supported: ${algorithm}`);
}

export function getCiphers() { return []; }
export function getHashes() { return ["sha256"]; }

export default { randomBytes, randomUUID, randomFillSync, createHash, getCiphers, getHashes };
