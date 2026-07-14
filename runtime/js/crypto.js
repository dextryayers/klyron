// Klyron Runtime — Web Crypto API subset
// crypto.getRandomValues, crypto.randomUUID, SubtleCrypto stubs

class SubtleCrypto {
  async digest(algorithm, data) {
    const algo = typeof algorithm === 'string' ? algorithm.toUpperCase() : algorithm.name.toUpperCase();
    const buf = data instanceof ArrayBuffer ? new Uint8Array(data) : new Uint8Array(data);
    return Klyron.crypto.digest(algo, buf).buffer;
  }

  async randomUUID() { return Klyron.crypto.randomUUID(); }
}

class Crypto {
  constructor() {
    this.subtle = new SubtleCrypto();
  }

  getRandomValues(array) {
    if (!(array instanceof Uint8Array || array instanceof Int8Array ||
          array instanceof Uint16Array || array instanceof Int16Array ||
          array instanceof Uint32Array || array instanceof Int32Array)) {
      throw new TypeError('Expected integer TypedArray');
    }
    const bytes = Klyron.crypto.randomBytes(array.byteLength);
    array.set(new Uint8Array(bytes));
    return array;
  }

  randomUUID() {
    return Klyron.crypto.randomUUID();
  }
}

if (typeof globalThis.crypto !== 'object') {
  globalThis.crypto = new Crypto();
}
