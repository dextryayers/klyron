// Klyron Runtime — node:buffer polyfill

const enc = globalThis.TextEncoder || class {
  encode(s) {
    const u = new Uint8Array(s.length * 3);
    let len = 0;
    for (let i = 0; i < s.length; i++) {
      let c = s.charCodeAt(i);
      if (c < 0x80) u[len++] = c;
      else if (c < 0x800) { u[len++] = 0xC0 | (c >> 6); u[len++] = 0x80 | (c & 0x3F); }
      else { u[len++] = 0xE0 | (c >> 12); u[len++] = 0x80 | ((c >> 6) & 0x3F); u[len++] = 0x80 | (c & 0x3F); }
    }
    return u.slice(0, len);
  }
};
const dec = globalThis.TextDecoder || class {
  constructor(l = 'utf-8') { this._l = l.toLowerCase(); }
  get encoding() { return this._l; }
  decode(b) {
    const bytes = b instanceof Uint8Array ? b : new Uint8Array(b);
    let out = '';
    for (let i = 0; i < bytes.length; i++) {
      if (bytes[i] < 0x80) out += String.fromCharCode(bytes[i]);
      else if (bytes[i] < 0xE0 && i + 1 < bytes.length) {
        out += String.fromCharCode((bytes[i] & 0x1F) << 6 | (bytes[i + 1] & 0x3F)); i++;
      } else if (bytes[i] < 0xF0 && i + 2 < bytes.length) {
        out += String.fromCharCode((bytes[i] & 0x0F) << 12 | (bytes[i + 1] & 0x3F) << 6 | (bytes[i + 2] & 0x3F)); i += 2;
      } else i++;
    }
    return out;
  }
};
const btoaFn = globalThis.btoa;
const atobFn = globalThis.atob;

class Buffer extends Uint8Array {
  constructor(arg, encodingOrOffset, length) {
    if (typeof arg === 'number') {
      super(arg);
    } else if (typeof arg === 'string') {
      const enc = encodingOrOffset || 'utf8';
      if (enc === 'hex') {
        super(arg.length / 2);
        for (let i = 0; i < arg.length; i += 2) this[i/2] = parseInt(arg.substr(i, 2), 16);
      } else if (enc === 'base64') {
        const bin = atobFn(arg);
        super(bin.length);
        for (let i = 0; i < bin.length; i++) this[i] = bin.charCodeAt(i);
      } else if (enc === 'base64url') {
        const bin = atobFn(arg.replace(/-/g, '+').replace(/_/g, '/'));
        super(bin.length);
        for (let i = 0; i < bin.length; i++) this[i] = bin.charCodeAt(i);
      } else {
        const u8 = new enc().encode(arg);
        super(u8);
      }
    } else if (arg instanceof ArrayBuffer || ArrayBuffer.isView(arg)) {
      super(arg);
    } else if (arg && typeof arg === 'object' && typeof arg.length === 'number') {
      super(arg.length);
      for (let i = 0; i < arg.length; i++) this[i] = arg[i];
    } else {
      super(arg);
    }
  }

  toString(encoding = 'utf8', start = 0, end = this.length) {
    if (start < 0) start = 0;
    if (end > this.length) end = this.length;
    const slice = this.subarray(start, end);
    switch (encoding) {
      case 'hex':
        return Array.from(slice).map(b => b.toString(16).padStart(2, '0')).join('');
      case 'base64':
        return btoaFn(String.fromCharCode(...slice));
      case 'base64url':
        return btoaFn(String.fromCharCode(...slice)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
      case 'ascii':
        return String.fromCharCode(...slice);
      case 'ucs2':
      case 'ucs-2':
      case 'utf16le':
      case 'utf-16le': {
        let out = '';
        for (let i = 0; i < slice.length; i += 2) {
          out += String.fromCharCode(slice[i] | (slice[i + 1] << 8));
        }
        return out;
      }
      case 'latin1':
      case 'binary':
        return String.fromCharCode(...slice);
      default:
        return new dec().decode(slice);
    }
  }

  toJSON() { return { type: 'Buffer', data: Array.from(this) }; }

  slice(start = 0, end = this.length) {
    if (start < 0) start = this.length + start;
    if (end < 0) end = this.length + end;
    if (start < 0) start = 0;
    if (end > this.length) end = this.length;
    return Buffer.from(super.slice(start, end));
  }

  subarray(start = 0, end = this.length) {
    return Buffer.from(super.subarray(start, end));
  }

  equals(other) {
    if (this.length !== other.length) return false;
    for (let i = 0; i < this.length; i++) {
      if (this[i] !== other[i]) return false;
    }
    return true;
  }

  compare(other) {
    const len = Math.min(this.length, other.length);
    for (let i = 0; i < len; i++) {
      if (this[i] < other[i]) return -1;
      if (this[i] > other[i]) return 1;
    }
    return this.length - other.length;
  }

  copy(target, targetStart = 0, sourceStart = 0, sourceEnd = this.length) {
    const copied = Math.min(sourceEnd - sourceStart, target.length - targetStart);
    for (let i = 0; i < copied; i++) {
      target[targetStart + i] = this[sourceStart + i];
    }
    return copied;
  }

  write(str, offset = 0, length = this.length - offset, encoding = 'utf8') {
    if (encoding === 'hex') {
      str = str.replace(/^0x/i, '');
      const len = Math.min(str.length / 2, length);
      for (let i = 0; i < len; i++) {
        this[offset + i] = parseInt(str.substr(i * 2, 2), 16);
      }
      return len;
    }
    if (encoding === 'base64') {
      const bin = atobFn(str);
      const len = Math.min(bin.length, length);
      for (let i = 0; i < len; i++) this[offset + i] = bin.charCodeAt(i);
      return len;
    }
    const bytes = new enc().encode(str).slice(0, Math.min(length, this.length - offset));
    this.set(bytes, offset);
    return bytes.length;
  }

  readInt8(offset = 0) {
    if (offset >= this.length) throw new RangeError('Index out of range');
    return this[offset] << 24 >> 24;
  }

  readUInt8(offset = 0) {
    if (offset >= this.length) throw new RangeError('Index out of range');
    return this[offset];
  }

  readInt16LE(offset = 0) {
    if (offset + 1 >= this.length) throw new RangeError('Index out of range');
    return (this[offset] | (this[offset + 1] << 8)) << 16 >> 16;
  }

  readInt16BE(offset = 0) {
    if (offset + 1 >= this.length) throw new RangeError('Index out of range');
    return ((this[offset] << 8) | this[offset + 1]) << 16 >> 16;
  }

  readUInt16LE(offset = 0) {
    if (offset + 1 >= this.length) throw new RangeError('Index out of range');
    return this[offset] | (this[offset + 1] << 8);
  }

  readUInt16BE(offset = 0) {
    if (offset + 1 >= this.length) throw new RangeError('Index out of range');
    return (this[offset] << 8) | this[offset + 1];
  }

  readInt32LE(offset = 0) {
    if (offset + 3 >= this.length) throw new RangeError('Index out of range');
    return this[offset] | (this[offset + 1] << 8) | (this[offset + 2] << 16) | (this[offset + 3] << 24);
  }

  readInt32BE(offset = 0) {
    if (offset + 3 >= this.length) throw new RangeError('Index out of range');
    return (this[offset] << 24) | (this[offset + 1] << 16) | (this[offset + 2] << 8) | this[offset + 3];
  }

  readUInt32LE(offset = 0) {
    if (offset + 3 >= this.length) throw new RangeError('Index out of range');
    return ((this[offset] | (this[offset + 1] << 8) | (this[offset + 2] << 16) | (this[offset + 3] << 24)) >>> 0);
  }

  readUInt32BE(offset = 0) {
    if (offset + 3 >= this.length) throw new RangeError('Index out of range');
    return (((this[offset] << 24) | (this[offset + 1] << 16) | (this[offset + 2] << 8) | this[offset + 3]) >>> 0);
  }

  readFloatLE(offset = 0) {
    const arr = new Uint8Array(4);
    for (let i = 0; i < 4; i++) arr[i] = this[offset + i] || 0;
    return new Float32Array(arr.buffer)[0];
  }

  readFloatBE(offset = 0) {
    const arr = new Uint8Array(4);
    for (let i = 0; i < 4; i++) arr[3 - i] = this[offset + i] || 0;
    return new Float32Array(arr.buffer)[0];
  }

  readDoubleLE(offset = 0) {
    const arr = new Uint8Array(8);
    for (let i = 0; i < 8; i++) arr[i] = this[offset + i] || 0;
    return new Float64Array(arr.buffer)[0];
  }

  readDoubleBE(offset = 0) {
    const arr = new Uint8Array(8);
    for (let i = 0; i < 8; i++) arr[7 - i] = this[offset + i] || 0;
    return new Float64Array(arr.buffer)[0];
  }

  writeInt8(value, offset = 0) {
    if (offset >= this.length) throw new RangeError('Index out of range');
    this[offset] = value & 0xFF;
    return offset + 1;
  }

  writeUInt8(value, offset = 0) {
    return this.writeInt8(value, offset);
  }

  writeInt16LE(value, offset = 0) {
    if (offset + 1 >= this.length) throw new RangeError('Index out of range');
    this[offset] = value & 0xFF;
    this[offset + 1] = (value >> 8) & 0xFF;
    return offset + 2;
  }

  writeInt16BE(value, offset = 0) {
    if (offset + 1 >= this.length) throw new RangeError('Index out of range');
    this[offset] = (value >> 8) & 0xFF;
    this[offset + 1] = value & 0xFF;
    return offset + 2;
  }

  writeUInt16LE(value, offset = 0) { return this.writeInt16LE(value, offset); }
  writeUInt16BE(value, offset = 0) { return this.writeInt16BE(value, offset); }

  writeInt32LE(value, offset = 0) {
    if (offset + 3 >= this.length) throw new RangeError('Index out of range');
    this[offset] = value & 0xFF;
    this[offset + 1] = (value >> 8) & 0xFF;
    this[offset + 2] = (value >> 16) & 0xFF;
    this[offset + 3] = (value >> 24) & 0xFF;
    return offset + 4;
  }

  writeInt32BE(value, offset = 0) {
    if (offset + 3 >= this.length) throw new RangeError('Index out of range');
    this[offset] = (value >> 24) & 0xFF;
    this[offset + 1] = (value >> 16) & 0xFF;
    this[offset + 2] = (value >> 8) & 0xFF;
    this[offset + 3] = value & 0xFF;
    return offset + 4;
  }

  writeUInt32LE(value, offset = 0) { return this.writeInt32LE(value >>> 0, offset); }
  writeUInt32BE(value, offset = 0) { return this.writeInt32BE(value >>> 0, offset); }

  writeFloatLE(value, offset = 0) {
    const arr = new Float32Array([value]);
    const bytes = new Uint8Array(arr.buffer);
    for (let i = 0; i < 4; i++) this[offset + i] = bytes[i];
    return offset + 4;
  }

  writeFloatBE(value, offset = 0) {
    const arr = new Float32Array([value]);
    const bytes = new Uint8Array(arr.buffer);
    for (let i = 0; i < 4; i++) this[offset + i] = bytes[3 - i];
    return offset + 4;
  }

  writeDoubleLE(value, offset = 0) {
    const arr = new Float64Array([value]);
    const bytes = new Uint8Array(arr.buffer);
    for (let i = 0; i < 8; i++) this[offset + i] = bytes[i];
    return offset + 8;
  }

  writeDoubleBE(value, offset = 0) {
    const arr = new Float64Array([value]);
    const bytes = new Uint8Array(arr.buffer);
    for (let i = 0; i < 8; i++) this[offset + i] = bytes[7 - i];
    return offset + 8;
  }

  fill(value, offset = 0, end = this.length) {
    if (typeof value === 'string') value = value.charCodeAt(0);
    for (let i = offset; i < end; i++) this[i] = value;
    return this;
  }

  indexOf(value, byteOffset = 0) {
    if (typeof value === 'number') {
      for (let i = byteOffset; i < this.length; i++) if (this[i] === value) return i;
    } else if (typeof value === 'string') {
      const buf = Buffer.from(value);
      for (let i = byteOffset; i <= this.length - buf.length; i++) {
        let found = true;
        for (let j = 0; j < buf.length; j++) { if (this[i + j] !== buf[j]) { found = false; break; } }
        if (found) return i;
      }
    } else if (value instanceof Uint8Array) {
      for (let i = byteOffset; i <= this.length - value.length; i++) {
        let found = true;
        for (let j = 0; j < value.length; j++) { if (this[i + j] !== value[j]) { found = false; break; } }
        if (found) return i;
      }
    }
    return -1;
  }

  includes(value, byteOffset = 0) {
    return this.indexOf(value, byteOffset) !== -1;
  }

  entries() { return super.entries(); }
  keys() { return super.keys(); }
  values() { return super.values(); }

  // Static methods
  static from(value, encodingOrOffset, length) {
    return new Buffer(value, encodingOrOffset, length);
  }

  static alloc(size, fill = 0, encoding = 'utf8') {
    const b = new Buffer(size);
    if (fill !== 0) {
      if (typeof fill === 'string') {
        const bytes = new enc().encode(fill);
        for (let i = 0; i < size; i++) b[i] = bytes[i % bytes.length];
      } else {
        b.fill(fill);
      }
    }
    return b;
  }

  static allocUnsafe(size) { return new Buffer(size); }
  static allocUnsafeSlow(size) { return new Buffer(size); }

  static byteLength(str, encoding = 'utf8') {
    if (typeof str === 'string') {
      if (encoding === 'hex') return str.length / 2;
      if (encoding === 'base64' || encoding === 'base64url') {
        const len = str.length;
        if (len === 0) return 0;
        const padding = str.endsWith('=') ? (str.endsWith('==') ? 2 : 1) : 0;
        return Math.floor(len * 6 / 8) - padding;
      }
      return new enc().encode(str).length;
    }
    if (str instanceof Buffer || str instanceof Uint8Array) return str.length;
    if (ArrayBuffer.isView(str)) return str.byteLength;
    if (str instanceof ArrayBuffer) return str.byteLength;
    return str.length;
  }

  static isBuffer(obj) { return obj instanceof Buffer; }

  static isEncoding(encoding) {
    if (!encoding) return false;
    return ['utf8', 'utf-8', 'utf16le', 'utf-16le', 'ucs2', 'ucs-2',
            'latin1', 'binary', 'base64', 'base64url', 'hex', 'ascii'].includes(encoding.toLowerCase());
  }

  static concat(list, totalLength) {
    if (!totalLength) totalLength = list.reduce((s, b) => s + b.length, 0);
    const buf = Buffer.alloc(totalLength);
    let offset = 0;
    for (const b of list) { buf.set(b, offset); offset += b.length; }
    return buf;
  }

  static compare(buf1, buf2) {
    if (!Buffer.isBuffer(buf1)) buf1 = Buffer.from(buf1);
    if (!Buffer.isBuffer(buf2)) buf2 = Buffer.from(buf2);
    return buf1.compare(buf2);
  }

  static poolSize = 8192;

  static INSPECT_MAX_BYTES = 50;

  static fromString(string, encoding) {
    return Buffer.from(string, encoding);
  }
}

Object.defineProperty(Buffer.prototype, 'parent', { get() { return this.buffer; } });
Object.defineProperty(Buffer.prototype, 'offset', { get() { return this.byteOffset; } });

const kMaxLength = 0x7FFFFFFF;
const kStringMaxLength = 0x7FFFFFFF;

const bufferModule = {
  Buffer,
  INSPECT_MAX_BYTES: 50,
  kMaxLength,
  kStringMaxLength,
  atob: atobFn,
  btoa: btoaFn,
  Blob: globalThis.Blob || class Blob {
    constructor(parts = [], opts = {}) {
      this._parts = parts;
      this._type = opts.type || '';
    }
    get size() {
      return this._parts.reduce((s, p) => {
        if (typeof p === 'string') return s + new TextEncoder().encode(p).length;
        if (p instanceof Uint8Array) return s + p.length;
        if (p instanceof ArrayBuffer) return s + p.byteLength;
        return s;
      }, 0);
    }
    get type() { return this._type; }
    async text() {
      const enc = new TextDecoder();
      let result = '';
      for (const p of this._parts) {
        if (typeof p === 'string') result += p;
        else if (p instanceof Uint8Array) result += enc.decode(p);
        else if (p instanceof ArrayBuffer) result += enc.decode(new Uint8Array(p));
      }
      return result;
    }
    async arrayBuffer() {
      const enc = new TextEncoder();
      const strings = [];
      const bufs = [];
      for (const p of this._parts) {
        if (typeof p === 'string') strings.push(enc.encode(p));
        else if (p instanceof Uint8Array) bufs.push(p);
        else if (p instanceof ArrayBuffer) bufs.push(new Uint8Array(p));
      }
      const all = [...strings, ...bufs];
      const total = all.reduce((s, b) => s + b.length, 0);
      const result = new Uint8Array(total);
      let offset = 0;
      for (const b of all) { result.set(b, offset); offset += b.length; }
      return result.buffer;
    }
  },
  resolveObjectURL: () => {},
  File: globalThis.File || class File {
    constructor(parts, name, opts = {}) {
      this._blob = new Blob(parts, opts);
      this._name = name;
      this._lastModified = opts.lastModified || Date.now();
    }
    get name() { return this._name; }
    get lastModified() { return this._lastModified; }
    get size() { return this._blob.size; }
    get type() { return this._blob.type; }
    async text() { return this._blob.text(); }
    async arrayBuffer() { return this._blob.arrayBuffer(); }
  },
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = bufferModule;
}
