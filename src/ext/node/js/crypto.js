import { op_crypto_random_uuid, op_crypto_random_values, op_crypto_sha256, op_crypto_hex_encode, op_crypto_digest, op_crypto_hmac } from "ext:core/ops";

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

const textEncoder = new TextDecoder();

function hexEncode(data) {
  if (typeof data === "string") {
    const bytes = [];
    for (let i = 0; i < data.length; i++) bytes.push(data.charCodeAt(i));
    return op_crypto_hex_encode(bytes);
  }
  const arr = data instanceof Uint8Array ? Array.from(data) : [...data];
  return op_crypto_hex_encode(arr);
}

class Hash {
  constructor(algorithm) {
    this._algorithm = algorithm;
    this._data = [];
  }
  update(data, enc) {
    if (typeof data === "string") {
      this._data.push(...new TextEncoder().encode(data));
    } else {
      this._data.push(...new Uint8Array(data));
    }
    return this;
  }
  digest(enc) {
    const bytes = this._data;
    const hex = op_crypto_digest(this._algorithm, bytes);
    if (enc === "hex") return hex;
    if (enc === "base64" || enc === "base64url") {
      const binary = hex.match(/../g).map(b => String.fromCharCode(parseInt(b, 16))).join("");
      return btoa(binary);
    }
    return Buffer.from(hex, "hex");
  }
}

export function createHash(algorithm) {
  const algo = algorithm?.toLowerCase();
  const supported = ["md5", "sha1", "sha224", "sha256", "sha384", "sha512"];
  if (supported.includes(algo)) return new Hash(algo);
  throw new Error(`Hash algorithm not supported: ${algorithm}`);
}

class Hmac {
  constructor(algorithm, key) {
    this._algorithm = algorithm;
    this._key = key instanceof Uint8Array ? key : new TextEncoder().encode(String(key));
    this._data = [];
  }
  update(data, enc) {
    if (typeof data === "string") {
      this._data.push(...new TextEncoder().encode(data));
    } else {
      this._data.push(...new Uint8Array(data));
    }
    return this;
  }
  digest(enc) {
    const algoMap = { md5: "MD5", sha1: "SHA1", sha224: "SHA224", sha256: "SHA256", sha384: "SHA384", sha512: "SHA512" };
    const algo = algoMap[this._algorithm] || this._algorithm.toUpperCase();
    const keyHex = hexEncode(this._key);
    const dataHex = hexEncode(this._data);
    const hex = op_crypto_hmac(algo, keyHex, dataHex);
    if (enc === "hex") return hex;
    if (enc === "base64" || enc === "base64url") {
      const binary = hex.match(/../g).map(b => String.fromCharCode(parseInt(b, 16))).join("");
      return btoa(binary);
    }
    return Buffer.from(hex, "hex");
  }
}

export function createHmac(algorithm, key) {
  return new Hmac(algorithm, key);
}

export function getCiphers() { return []; }

export function getHashes() { return ["md5", "sha1", "sha224", "sha256", "sha384", "sha512"]; }

export default { randomBytes, randomUUID, randomFillSync, createHash, createHmac, getCiphers, getHashes };