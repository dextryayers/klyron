import {
  op_crypto_random_uuid, op_crypto_random_values, op_crypto_sha256,
  op_crypto_hex_encode, op_crypto_digest, op_crypto_encrypt, op_crypto_decrypt,
  op_crypto_sha512, op_crypto_hmac
} from "ext:core/ops";

export function randomUUID() { return op_crypto_random_uuid(); }
export function getRandomValues(buf) {
  const hex = op_crypto_random_values(String(buf.length));
  const bytes = [];
  for (let i = 0; i < hex.length; i += 2) bytes.push(parseInt(hex.slice(i, i + 2), 16));
  for (let i = 0; i < buf.length; i++) buf[i] = bytes[i] || 0;
  return buf;
}
export function sha256(data) { return op_crypto_sha256(data); }
export function hexEncode(data) {
  const arr = data instanceof Uint8Array ? Array.from(data) : [...data];
  return op_crypto_hex_encode(arr);
}

class SubtleCrypto {
  async digest(algorithm, data) {
    const algo = typeof algorithm === 'string' ? algorithm : algorithm.name;
    const bytes = data instanceof ArrayBuffer ? new Uint8Array(data) : new Uint8Array(data);
    const hex = op_crypto_digest(algo, Array.from(bytes));
    const hashBytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) hashBytes[i / 2] = parseInt(hex.slice(i, i + 2), 16);
    return hashBytes.buffer;
  }

  async encrypt(algorithm, key, data) {
    const algo = typeof algorithm === 'string' ? algorithm : algorithm.name;
    const bytes = data instanceof ArrayBuffer ? new Uint8Array(data) : new Uint8Array(data);
    const keyStr = key instanceof ArrayBuffer ? new Uint8Array(key) : new Uint8Array(key);
    const result = op_crypto_encrypt(algo, Array.from(bytes), Array.from(keyStr).map(b => String.fromCharCode(b)).join(''));
    const resultBytes = new Uint8Array(result.length / 2);
    for (let i = 0; i < result.length; i += 2) resultBytes[i / 2] = parseInt(result.slice(i, i + 2), 16);
    return resultBytes.buffer;
  }

  async decrypt(algorithm, key, data) {
    const algo = typeof algorithm === 'string' ? algorithm : algorithm.name;
    const bytes = data instanceof ArrayBuffer ? new Uint8Array(data) : new Uint8Array(data);
    const keyStr = key instanceof ArrayBuffer ? new Uint8Array(key) : new Uint8Array(key);
    const result = op_crypto_decrypt(algo, Array.from(bytes), Array.from(keyStr).map(b => String.fromCharCode(b)).join(''));
    const resultBytes = new Uint8Array(result.length / 2);
    for (let i = 0; i < result.length; i += 2) resultBytes[i / 2] = parseInt(result.slice(i, i + 2), 16);
    return resultBytes.buffer;
  }
}

export const subtle = new SubtleCrypto();

export default { randomUUID, getRandomValues, sha256, hexEncode, subtle };
