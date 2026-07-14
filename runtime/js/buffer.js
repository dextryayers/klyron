// Klyron Runtime — Buffer (Node.js compatible)
// Buffer, Blob, TextEncoder, TextDecoder

if (typeof globalThis.TextEncoder !== 'function') {
  globalThis.TextEncoder = class TextEncoder {
    encode(str = '') {
      const u8 = new Uint8Array(str.length * 3);
      let len = 0;
      for (let i = 0; i < str.length; i++) {
        let c = str.charCodeAt(i);
        if (c < 0x80) { u8[len++] = c; }
        else if (c < 0x800) { u8[len++] = 0xC0 | (c >> 6); u8[len++] = 0x80 | (c & 0x3F); }
        else { u8[len++] = 0xE0 | (c >> 12); u8[len++] = 0x80 | ((c >> 6) & 0x3F); u8[len++] = 0x80 | (c & 0x3F); }
      }
      return u8.slice(0, len);
    }
    get encoding() { return 'utf-8'; }
  };
}

if (typeof globalThis.TextDecoder !== 'function') {
  globalThis.TextDecoder = class TextDecoder {
    constructor(label = 'utf-8') { this._label = label.toLowerCase(); }
    get encoding() { return this._label; }
    decode(buf, opts = {}) {
      const bytes = buf instanceof Uint8Array ? buf : new Uint8Array(buf);
      let out = '';
      for (let i = 0; i < bytes.length; i++) {
        if (bytes[i] < 0x80) { out += String.fromCharCode(bytes[i]); }
        else if (bytes[i] < 0xE0 && i + 1 < bytes.length) {
          out += String.fromCharCode((bytes[i] & 0x1F) << 6 | (bytes[i + 1] & 0x3F));
          i++;
        } else if (bytes[i] < 0xF0 && i + 2 < bytes.length) {
          out += String.fromCharCode((bytes[i] & 0x0F) << 12 | (bytes[i + 1] & 0x3F) << 6 | (bytes[i + 2] & 0x3F));
          i += 2;
        } else { i++; }
      }
      if (opts.stream !== true && bytes.length > 0 && out.length === 0) return '';
      return out;
    }
  };
}

if (typeof globalThis.Buffer === 'undefined' || typeof globalThis.Buffer !== 'function') {
  class Buffer extends Uint8Array {
    constructor(arg, encodingOrOffset, length) {
      if (typeof arg === 'number') super(arg);
      else if (typeof arg === 'string') {
        const enc = encodingOrOffset || 'utf8';
        if (enc === 'hex') { super(arg.length / 2); for (let i = 0; i < arg.length; i += 2) this[i/2] = parseInt(arg.substr(i, 2), 16); }
        else if (enc === 'base64') { const bin = atob(arg); super(bin.length); for (let i = 0; i < bin.length; i++) this[i] = bin.charCodeAt(i); }
        else { const enc = new TextEncoder(); const u8 = enc.encode(arg); super(u8); }
      } else { super(arg); }
    }
    toString(encoding = 'utf8', start = 0, end = this.length) {
      if (encoding === 'hex') return Array.from(this.slice(start, end)).map(b => b.toString(16).padStart(2, '0')).join('');
      if (encoding === 'base64') return btoa(String.fromCharCode(...this.slice(start, end)));
      if (encoding === 'utf8' || encoding === 'utf-8') return new TextDecoder().decode(this.slice(start, end));
      return new TextDecoder(encoding).decode(this.slice(start, end));
    }
    toJSON() { return { type: 'Buffer', data: Array.from(this) }; }
    equals(other) { return this.length === other.length && this.every((v, i) => v === other[i]); }
    copy(target, targetStart = 0, sourceStart = 0, sourceEnd = this.length) {
      const copied = Math.min(sourceEnd - sourceStart, target.length - targetStart);
      for (let i = 0; i < copied; i++) target[targetStart + i] = this[sourceStart + i];
      return copied;
    }
    slice(start = 0, end = this.length) { return Buffer.from(super.slice(start, end)); }
    static from(value, encodingOrOffset, length) { return new Buffer(value, encodingOrOffset, length); }
    static alloc(size, fill = 0) {
      const b = new Buffer(size);
      if (fill !== 0) b.fill(fill);
      return b;
    }
    static allocUnsafe(size) { return new Buffer(size); }
    static byteLength(str, encoding = 'utf8') {
      if (typeof str === 'string') return new TextEncoder().encode(str).length;
      return str.length;
    }
    static isBuffer(obj) { return obj instanceof Buffer; }
    static concat(list, totalLength) {
      if (!totalLength) totalLength = list.reduce((s, b) => s + b.length, 0);
      const buf = Buffer.alloc(totalLength);
      let offset = 0;
      for (const b of list) { buf.set(b, offset); offset += b.length; }
      return buf;
    }
    static isEncoding(enc) { return ['utf8', 'utf-8', 'hex', 'base64', 'ascii', 'latin1'].includes(enc?.toLowerCase()); }
    entries() { return super.entries(); }
    keys() { return super.keys(); }
    values() { return super.values(); }
    write(str, offset = 0, length = str.length, encoding = 'utf8') {
      const bytes = new TextEncoder().encode(str).slice(0, Math.min(length, this.length - offset));
      this.set(bytes, offset);
      return bytes.length;
    }
  }

  Object.defineProperty(Buffer.prototype, 'parent', { get() { return this.buffer; } });
  Object.defineProperty(Buffer.prototype, 'offset', { get() { return this.byteOffset; } });

  globalThis.Buffer = Buffer;
}
