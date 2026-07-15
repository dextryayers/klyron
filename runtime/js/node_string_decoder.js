// Klyron Runtime — node:string_decoder polyfill

const kIncompleteCharactersMap = new Map([
  ['utf8', { 0: { type: 'none', bytes: 0 }, 1: { type: 'none', bytes: 0 }, 2: { type: 'none', bytes: 0 } }],
  ['utf-8', { 0: { type: 'none', bytes: 0 }, 1: { type: 'none', bytes: 0 }, 2: { type: 'none', bytes: 0 } }],
]);

function utf8CheckByte(byte) {
  if (byte <= 0x7F) return 0;
  if (byte >> 5 === 0x06) return 2;
  if (byte >> 4 === 0x0E) return 3;
  if (byte >> 3 === 0x1E) return 4;
  return -1;
}

function utf8CheckIncomplete(self, buf, i) {
  let j = buf.length - 1;
  if (j < i) return 0;
  let nb = utf8CheckByte(buf[j]);
  if (nb >= 0) {
    if (nb > 0) self.lastNeed = nb - 1;
    return nb;
  }
  if (--j < i || nb === -1) return 0;
  nb = utf8CheckByte(buf[j]);
  if (nb >= 0) {
    if (nb > 0) self.lastNeed = nb - 2;
    return nb;
  }
  if (--j < i || nb === -1) return 0;
  nb = utf8CheckByte(buf[j]);
  if (nb >= 0) {
    if (nb > 0) {
      if (nb === 2) nb = 0; else self.lastNeed = nb - 3;
    }
    return nb;
  }
  return 0;
}

function utf8FillIncomplete(self, buf) {
  if (self.lastNeed > 0) {
    const bufLen = buf.length;
    if (self.lastNeed >= bufLen) {
      self.lastBuf = new Uint8Array(buf);
      self.lastNeed -= bufLen;
      return '';
    }
    const tbuf = new Uint8Array(self.lastBuf.length + bufLen);
    tbuf.set(self.lastBuf);
    tbuf.set(buf, self.lastBuf.length);
    const result = new TextDecoder().decode(tbuf.slice(0, tbuf.length - self.lastNeed));
    self.lastNeed = 0;
    self.lastBuf = null;
    return result;
  }
  return '';
}

class StringDecoder {
  constructor(encoding = 'utf8') {
    this.encoding = encoding.toLowerCase();
    this.lastNeed = 0;
    this.lastTotal = 0;
    this.lastChar = Buffer ? Buffer.alloc(4) : new Uint8Array(4);
    this.lastBuf = null;
  }

  write(buffer) {
    if (typeof buffer === 'string') return buffer;
    const buf = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
    const enc = this.encoding;
    if (enc === 'utf8' || enc === 'utf-8') {
      let result = utf8FillIncomplete(this, buf);
      if (buf.length > 0) {
        const incomplete = utf8CheckIncomplete(this, buf, 0);
        let end = buf.length;
        if (incomplete >= 0) {
          end = buf.length - incomplete;
          if (end < 0) end = 0;
        }
        result += new TextDecoder().decode(buf.slice(0, end));
        if (incomplete >= 0 && end < buf.length) {
          this.lastNeed = buf.length - end;
          this.lastTotal = this.lastNeed;
          this.lastChar = new Uint8Array(buf.slice(end));
        }
      }
      return result;
    }
    if (enc === 'base64') {
      return btoa(String.fromCharCode(...buf));
    }
    if (enc === 'hex') {
      return Array.from(buf).map(b => b.toString(16).padStart(2, '0')).join('');
    }
    if (enc === 'latin1' || enc === 'ascii' || enc === 'binary') {
      return String.fromCharCode(...buf);
    }
    if (enc === 'utf16le' || enc === 'utf-16le') {
      let result = '';
      for (let i = 0; i < buf.length; i += 2) {
        result += String.fromCharCode(buf[i] | (buf[i + 1] << 8));
      }
      return result;
    }
    return new TextDecoder(this.encoding).decode(buf);
  }

  end(buffer) {
    if (buffer) {
      const result = this.write(buffer);
      if (this.lastNeed > 0 && this.lastBuf) {
        return result + new TextDecoder().decode(this.lastBuf);
      }
      return result;
    }
    if (this.lastNeed > 0 && this.lastBuf) {
      const result = new TextDecoder().decode(this.lastBuf);
      this.lastBuf = null;
      this.lastNeed = 0;
      return result;
    }
    return '';
  }

  text(buffer, offset) {
    const end = buffer.length;
    if (typeof offset !== 'number') offset = 0;
    const slice = buffer.slice(offset, end);
    return new TextDecoder(this.encoding).decode(slice);
  }

  lastChar() {
    if (this.lastChar) {
      return String.fromCharCode(this.lastChar[0] || 0);
    }
    return '';
  }
}

if (typeof module !== 'undefined' && module.exports) {
  module.exports = { StringDecoder };
}
