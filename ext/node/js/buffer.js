export class Buffer extends Uint8Array {
  constructor(size) { super(size); }
  static alloc(size, fill) {
    const b = new Buffer(size);
    if (fill !== undefined) b.fill(fill);
    return b;
  }
  static from(data, encoding) {
    if (typeof data === "string") {
      const encoder = new TextEncoder();
      return encoder.encode(data);
    }
    if (Array.isArray(data)) return new Buffer(data);
    if (data instanceof Uint8Array) return new Buffer(data);
    return new Buffer(0);
  }
  static concat(list, totalLength) {
    if (!totalLength) totalLength = list.reduce((s, b) => s + b.length, 0);
    const result = Buffer.alloc(totalLength);
    let offset = 0;
    for (const b of list) { result.set(b, offset); offset += b.length; }
    return result;
  }
  static isBuffer(obj) { return obj instanceof Buffer; }
  static byteLength(str, enc) {
    if (typeof str !== "string") return str.byteLength || str.length;
    return new TextEncoder().encode(str).length;
  }
  toString(encoding, start, end) {
    const slice = this.slice(start || 0, end || this.length);
    return new TextDecoder().decode(slice);
  }
  toJSON() { return { type: "Buffer", data: [...this] }; }
  write(str, offset, length, encoding) {
    const bytes = new TextEncoder().encode(str);
    for (let i = 0; i < bytes.length && i < (length || bytes.length); i++)
      this[offset + i] = bytes[i];
    return bytes.length;
  }
  fill(value, offset, end) {
    offset = offset || 0; end = end || this.length;
    for (let i = offset; i < end; i++) this[i] = typeof value === "number" ? value : (value?.charCodeAt(0) || 0);
    return this;
  }
  slice(start, end) { return new Buffer(super.slice(start, end)); }
  copy(target, targetStart, sourceStart, sourceEnd) {
    sourceStart = sourceStart || 0; sourceEnd = sourceEnd || this.length;
    const len = Math.min(sourceEnd - sourceStart, target.length - (targetStart || 0));
    for (let i = 0; i < len; i++) target[targetStart + i] = this[sourceStart + i];
    return len;
  }
  indexOf(value, byteOffset) {
    byteOffset = byteOffset || 0;
    const val = typeof value === "number" ? [value] : [...value];
    for (let i = byteOffset; i <= this.length - val.length; i++) {
      let match = true;
      for (let j = 0; j < val.length; j++) { if (this[i + j] !== val[j]) { match = false; break; } }
      if (match) return i;
    }
    return -1;
  }
  includes(value, byteOffset) { return this.indexOf(value, byteOffset) !== -1; }
  equals(other) {
    if (this.length !== other.length) return false;
    for (let i = 0; i < this.length; i++) if (this[i] !== other[i]) return false;
    return true;
  }
}

const kMaxLength = 0x7fffffff;
Buffer.kMaxLength = kMaxLength;
Buffer.allocUnsafe = Buffer.alloc;
Buffer.allocUnsafeSlow = Buffer.alloc;

export { kMaxLength };

export const INSPECT_MAX_BYTES = 50;

export default Buffer;
