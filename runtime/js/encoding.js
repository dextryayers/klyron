// Klyron Runtime — Encoding API (Web compatible)
// TextEncoder, TextDecoder are in buffer.js
// atob, btoa

if (typeof globalThis.btoa !== 'function') {
  globalThis.btoa = (str) => {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    let result = '';
    const bytes = new TextEncoder().encode(str);
    for (let i = 0; i < bytes.length; i += 3) {
      const b1 = bytes[i];
      const b2 = i + 1 < bytes.length ? bytes[i + 1] : 0;
      const b3 = i + 2 < bytes.length ? bytes[i + 2] : 0;
      result += chars[b1 >> 2];
      result += chars[((b1 & 3) << 4) | (b2 >> 4)];
      result += i + 1 < bytes.length ? chars[((b2 & 15) << 2) | (b3 >> 6)] : '=';
      result += i + 2 < bytes.length ? chars[b3 & 63] : '=';
    }
    return result;
  };
}

if (typeof globalThis.atob !== 'function') {
  globalThis.atob = (str) => {
    str = str.replace(/[^A-Za-z0-9+/=]/g, '');
    let result = '';
    for (let i = 0; i < str.length; i += 4) {
      const c1 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'.indexOf(str[i]);
      const c2 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'.indexOf(str[i + 1]);
      const c3 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'.indexOf(str[i + 2]);
      const c4 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'.indexOf(str[i + 3]);
      result += String.fromCharCode((c1 << 2) | (c2 >> 4));
      if (c3 >= 0) result += String.fromCharCode(((c2 & 15) << 4) | (c3 >> 2));
      if (c4 >= 0) result += String.fromCharCode(((c3 & 3) << 6) | c4);
    }
    return result;
  };
}
