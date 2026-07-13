import { op_crypto_random_uuid, op_crypto_random_values, op_crypto_sha256, op_crypto_hex_encode } from "ext:core/ops";

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

export default { randomUUID, getRandomValues, sha256, hexEncode };
