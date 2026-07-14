// Klyron Runtime — node:string_decoder polyfill

class StringDecoder {
  constructor(encoding = 'utf8') {
    this.encoding = encoding.toLowerCase();
    this._buffer = '';
  }

  write(buffer) {
    if (this.encoding === 'utf8' || this.encoding === 'utf-8') {
      const decoder = new TextDecoder('utf-8', { stream: true });
      const result = decoder.decode(buffer, { stream: true });
      return result;
    }
    if (this.encoding === 'base64') {
      if (typeof buffer === 'string') return buffer;
      return btoa(String.fromCharCode(...new Uint8Array(buffer)));
    }
    if (this.encoding === 'hex') {
      return Array.from(new Uint8Array(buffer)).map(b => b.toString(16).padStart(2, '0')).join('');
    }
    if (typeof buffer === 'string') return buffer;
    return new TextDecoder(this.encoding).decode(buffer);
  }

  end(buffer) {
    if (buffer) return this.write(buffer);
    return '';
  }

  text(buffer, start, end) {
    return new TextDecoder(this.encoding).decode(buffer.slice(start, end));
  }

  lastChar() {
    return '';
  }

  lastNeed() {
    return 0;
  }

  lastCharNeed() {
    return false;
  }
}

if (typeof module !== 'undefined' && module.exports) {
  module.exports = { StringDecoder };
}
