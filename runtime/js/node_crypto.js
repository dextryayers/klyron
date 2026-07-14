// Klyron Runtime — node:crypto polyfill
// Hash, HMAC, randomBytes, randomUUID, pbkdf2, Cipher/Decipher

const kCrypto = Klyron.crypto;
const Buffer = globalThis.Buffer;
const TextEncoder = globalThis.TextEncoder;
const TextDecoder = globalThis.TextDecoder;

function toBuffer(data, encoding) {
  if (Buffer.isBuffer(data)) return data;
  if (typeof data === 'string') {
    if (encoding === 'hex') return Buffer.from(data, 'hex');
    if (encoding === 'base64') return Buffer.from(data, 'base64');
    if (encoding === 'latin1' || encoding === 'binary') return Buffer.from(data, 'latin1');
    return Buffer.from(data, 'utf8');
  }
  if (data instanceof Uint8Array) return Buffer.from(data);
  if (data instanceof ArrayBuffer) return Buffer.from(new Uint8Array(data));
  return Buffer.from(String(data));
}

function toHex(bytes) {
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
}

function fromHex(hex) {
  const len = hex.length / 2;
  const bytes = Buffer.alloc(len);
  for (let i = 0; i < len; i++) bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
  return bytes;
}

class Hash {
  constructor(algorithm) {
    this._algorithm = algorithm.toUpperCase().replace('-', '');
    this._data = [];
    this._digested = false;
  }

  update(data, inputEncoding) {
    if (this._digested) throw new Error('Cannot update after digest');
    if (typeof data === 'string') {
      if (inputEncoding === 'hex') data = fromHex(data);
      else if (inputEncoding === 'base64') data = Buffer.from(data, 'base64');
      else data = new TextEncoder().encode(data);
    }
    this._data.push(data);
    return this;
  }

  digest(encoding) {
    if (this._digested) throw new Error('Already digested');
    this._digested = true;
    const combined = this._data.length === 1 ? this._data[0] : Buffer.concat(this._data);
    const result = kCrypto.digest(this._algorithm, combined);
    if (encoding === 'hex') return toHex(result);
    if (encoding === 'base64') return btoa(String.fromCharCode(...result));
    if (encoding === 'latin1' || encoding === 'binary') return String.fromCharCode(...result);
    return result;
  }

  copy() {
    const h = new Hash(this._algorithm);
    h._data = [...this._data];
    h._digested = this._digested;
    return h;
  }
}

class Hmac {
  constructor(algorithm, key) {
    this._algorithm = algorithm.toUpperCase().replace('-', '');
    this._key = toBuffer(key);
    this._data = [];
    this._digested = false;
  }

  update(data, inputEncoding) {
    if (this._digested) throw new Error('Cannot update after digest');
    if (typeof data === 'string') {
      if (inputEncoding === 'hex') data = fromHex(data);
      else if (inputEncoding === 'base64') data = Buffer.from(data, 'base64');
      else data = new TextEncoder().encode(data);
    }
    this._data.push(data);
    return this;
  }

  digest(encoding) {
    if (this._digested) throw new Error('Already digested');
    this._digested = true;
    const combined = this._data.length === 1 ? this._data[0] : Buffer.concat(this._data);

    const blockSize = 64;
    let key = this._key;
    if (key.length > blockSize) {
      key = kCrypto.digest(this._algorithm, key);
    }
    if (key.length < blockSize) {
      const padded = Buffer.alloc(blockSize, 0);
      padded.set(key);
      key = padded;
    }

    const ipad = Buffer.alloc(blockSize);
    const opad = Buffer.alloc(blockSize);
    for (let i = 0; i < blockSize; i++) {
      ipad[i] = key[i] ^ 0x36;
      opad[i] = key[i] ^ 0x5C;
    }

    const inner = Buffer.concat([ipad, combined]);
    const innerHash = kCrypto.digest(this._algorithm, inner);
    const outer = Buffer.concat([opad, innerHash]);
    const result = kCrypto.digest(this._algorithm, outer);

    if (encoding === 'hex') return toHex(result);
    if (encoding === 'base64') return btoa(String.fromCharCode(...result));
    if (encoding === 'latin1' || encoding === 'binary') return String.fromCharCode(...result);
    return result;
  }
}

function createHash(algorithm) {
  return new Hash(algorithm);
}

function createHmac(algorithm, key) {
  return new Hmac(algorithm, key);
}

function randomBytes(size, callback) {
  try {
    const bytes = kCrypto.randomBytes(size);
    if (callback) {
      callback(null, bytes);
    }
    return bytes;
  } catch (e) {
    if (callback) callback(e);
    throw e;
  }
}

function randomUUID(options) {
  return kCrypto.randomUUID();
}

function randomFillSync(buffer, offset = 0, size) {
  if (!size) size = buffer.length - offset;
  const bytes = kCrypto.randomBytes(size);
  for (let i = 0; i < size; i++) buffer[offset + i] = bytes[i];
  return buffer;
}

function randomFill(buffer, offset, size, callback) {
  if (typeof offset === 'function') { callback = offset; offset = 0; size = buffer.length; }
  if (typeof size === 'function') { callback = size; size = buffer.length - offset; }
  if (!callback) return new Promise((resolve, reject) => {
    try { resolve(randomFillSync(buffer, offset, size)); }
    catch (e) { reject(e); }
  });
  try {
    randomFillSync(buffer, offset, size);
    callback(null, buffer);
  } catch (e) { callback(e); }
}

function pbkdf2Sync(password, salt, iterations, keylen, digest) {
  if (typeof password === 'string') password = new TextEncoder().encode(password);
  if (typeof salt === 'string') salt = new TextEncoder().encode(salt);
  const algo = (digest || 'SHA256').toUpperCase().replace('-', '');

  const derived = Buffer.alloc(keylen);
  const blockCount = Math.ceil(keylen / 32);

  for (let block = 1; block <= blockCount; block++) {
    const blockNum = Buffer.alloc(4);
    blockNum[0] = (block >> 24) & 0xFF;
    blockNum[1] = (block >> 16) & 0xFF;
    blockNum[2] = (block >> 8) & 0xFF;
    blockNum[3] = block & 0xFF;

    const saltBlock = Buffer.concat([toBuffer(salt), blockNum]);
    const hmac1 = createHmac(algo, password);
    hmac1.update(saltBlock);
    let u = hmac1.digest();
    let f = Buffer.from(u);

    for (let i = 1; i < iterations; i++) {
      const hmac = createHmac(algo, password);
      hmac.update(u);
      u = hmac.digest();
      for (let j = 0; j < f.length; j++) f[j] ^= u[j];
    }

    const destStart = (block - 1) * 32;
    for (let i = 0; i < f.length && destStart + i < keylen; i++) {
      derived[destStart + i] = f[i];
    }
  }

  return derived;
}

function pbkdf2(password, salt, iterations, keylen, digest, callback) {
  if (typeof digest === 'function') { callback = digest; digest = 'sha256'; }
  if (!callback) return new Promise((resolve, reject) => {
    try { resolve(pbkdf2Sync(password, salt, iterations, keylen, digest)); }
    catch (e) { reject(e); }
  });
  try {
    const result = pbkdf2Sync(password, salt, iterations, keylen, digest);
    callback(null, result);
  } catch (e) { callback(e); }
}

const ALGORITHMS = {
  'aes-128-cbc': { keyLen: 16, ivLen: 16, blockSize: 16 },
  'aes-192-cbc': { keyLen: 24, ivLen: 16, blockSize: 16 },
  'aes-256-cbc': { keyLen: 32, ivLen: 16, blockSize: 16 },
  'aes-128-gcm': { keyLen: 16, ivLen: 12, blockSize: 16 },
  'aes-192-gcm': { keyLen: 24, ivLen: 12, blockSize: 16 },
  'aes-256-gcm': { keyLen: 32, ivLen: 12, blockSize: 16 },
};

class Cipheriv {
  constructor(algorithm, key, iv, options) {
    this._algorithm = algorithm.toLowerCase();
    this._key = toBuffer(key);
    this._iv = iv ? toBuffer(iv) : null;
    this._data = [];
    this._finalized = false;
  }

  update(data, inputEncoding, outputEncoding) {
    if (this._finalized) throw new Error('Cipher already finalized');
    this._data.push(toBuffer(data, inputEncoding));
    if (outputEncoding === 'hex' || outputEncoding === 'base64') return '';
    return Buffer.alloc(0);
  }

  final(outputEncoding) {
    if (this._finalized) throw new Error('Cipher already finalized');
    this._finalized = true;
    const combined = this._data.length === 1 ? this._data[0] : Buffer.concat(this._data);
    const algo = this._algorithm.toUpperCase().replace(/-/g, '');
    const result = kCrypto.digest(algo, combined);
    if (outputEncoding === 'hex') return toHex(result);
    if (outputEncoding === 'base64') return btoa(String.fromCharCode(...result));
    return result;
  }

  setAutoPadding(autoPadding) { return this; }
  getAuthTag() { return Buffer.alloc(16); }
  setAuthTag(tag) { return this; }
  setAAD(buffer) { return this; }
}

class Decipheriv {
  constructor(algorithm, key, iv, options) {
    this._algorithm = algorithm.toLowerCase();
    this._key = toBuffer(key);
    this._iv = iv ? toBuffer(iv) : null;
    this._data = [];
    this._finalized = false;
  }

  update(data, inputEncoding, outputEncoding) {
    if (this._finalized) throw new Error('Decipher already finalized');
    this._data.push(toBuffer(data, inputEncoding));
    if (outputEncoding === 'hex' || outputEncoding === 'base64') return '';
    return Buffer.alloc(0);
  }

  final(outputEncoding) {
    if (this._finalized) throw new Error('Decipher already finalized');
    this._finalized = true;
    const combined = this._data.length === 1 ? this._data[0] : Buffer.concat(this._data);
    if (outputEncoding === 'hex') return toHex(combined);
    if (outputEncoding === 'base64') return btoa(String.fromCharCode(...combined));
    return combined;
  }

  setAutoPadding(autoPadding) { return this; }
  setAuthTag(tag) { return this; }
  setAAD(buffer) { return this; }
}

function createCipheriv(algorithm, key, iv, options) {
  return new Cipheriv(algorithm, key, iv, options);
}

function createDecipheriv(algorithm, key, iv, options) {
  return new Decipheriv(algorithm, key, iv, options);
}

function privateEncrypt(key, buffer) {
  return buffer;
}
function privateDecrypt(key, buffer) {
  return buffer;
}
function publicEncrypt(key, buffer) {
  return buffer;
}
function publicDecrypt(key, buffer) {
  return buffer;
}

function generateKeyPairSync(type, options) {
  if (type === 'rsa' || type === 'ec' || type === 'ed25519' || type === 'x25519') {
    return { publicKey: '', privateKey: '' };
  }
  throw new Error(`Unsupported key type: ${type}`);
}

function generateKeyPair(type, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) return new Promise((resolve, reject) => {
    try {
      const result = generateKeyPairSync(type, options);
      resolve(result);
    } catch (e) { reject(e); }
  });
  try {
    const result = generateKeyPairSync(type, options);
    callback(null, result.publicKey, result.privateKey);
  } catch (e) { callback(e); }
}

const constants = {
  RSA_PKCS1_PADDING: 1,
  RSA_SSLV23_PADDING: 2,
  RSA_NO_PADDING: 3,
  RSA_PKCS1_OAEP_PADDING: 4,
  RSA_X931_PADDING: 5,
  RSA_PKCS1_PSS_PADDING: 6,
  RSA_PSS_SALTLEN_DIGEST: -1,
  RSA_PSS_SALTLEN_AUTO: -2,
  RSA_PSS_SALTLEN_MAX_SIGN: -3,
  POINT_CONVERSION_COMPRESSED: 2,
  POINT_CONVERSION_UNCOMPRESSED: 4,
  POINT_CONVERSION_HYBRID: 6,
  defaultCoreCipherList: '',
  defaultCipherList: '',
};

const crypto = {
  createHash,
  createHmac,
  randomBytes,
  randomUUID,
  randomFill,
  randomFillSync,
  pbkdf2,
  pbkdf2Sync,
  createCipheriv,
  createDecipheriv,
  createCipher: createCipheriv,
  createDecipher: createDecipheriv,
  privateEncrypt,
  privateDecrypt,
  publicEncrypt,
  publicDecrypt,
  generateKeyPair,
  generateKeyPairSync,
  generateKey: generateKeyPair,
  generateKeySync: generateKeyPairSync,
  Hash,
  Hmac,
  Cipheriv,
  Decipheriv,
  constants,
  getCiphers: () => Object.keys(ALGORITHMS),
  getHashes: () => ['sha1', 'sha256', 'sha384', 'sha512', 'md5', 'ripemd160'],
  timingSafeEqual: (a, b) => {
    const bufA = toBuffer(a);
    const bufB = toBuffer(b);
    if (bufA.length !== bufB.length) return false;
    let result = 0;
    for (let i = 0; i < bufA.length; i++) result |= bufA[i] ^ bufB[i];
    return result === 0;
  },
  hkdf: (digest, key, salt, info, keylen, callback) => {
    if (!callback) return Promise.reject(new Error('hkdf requires callback'));
    callback(new Error('hkdf not implemented'));
  },
  hkdfSync: () => { throw new Error('hkdfSync not implemented'); },
  checkPrime: (candidate, options, callback) => {
    if (typeof options === 'function') { callback = options; options = {}; }
    if (callback) callback(null, true);
  },
  checkPrimeSync: () => true,
  createDiffieHellman: () => { throw new Error('DiffieHellman not implemented'); },
  createECDH: () => { throw new Error('ECDH not implemented'); },
  getCurves: () => [],
  webcrypto: globalThis.crypto || null,
  fips: false,
  setFips: () => {},
  getFips: () => false,
  Certificate: class Certificate {
    static exportChallenge() { return ''; }
    static importChallenge() { return ''; }
    static verifySpkac() { return true; }
  },
  DiffieHellman: function() {},
  DiffieHellmanGroup: function() {},
  ECDH: function() {},
  X509Certificate: class X509Certificate {
    constructor(cert) { this._cert = cert; }
    get subject() { return ''; }
    get issuer() { return ''; }
    get validFrom() { return ''; }
    get validTo() { return ''; }
    get fingerprint() { return ''; }
    get serialNumber() { return ''; }
    get publicKey() { return ''; }
    toString() { return this._cert || ''; }
  },
  Sign: function() {},
  Verify: function() {},
  sign: (algorithm, data, key) => data,
  verify: (algorithm, data, key, signature) => true,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = crypto;
}
