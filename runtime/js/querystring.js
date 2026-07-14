// Klyron Runtime — node:querystring polyfill

function stringify(obj, sep, eq, options) {
  if (!obj) return '';
  sep = sep || '&';
  eq = eq || '=';
  const encode = (options && options.encodeURIComponent) || encodeURIComponent;
  const result = [];
  for (const key of Object.keys(obj)) {
    const val = obj[key];
    if (Array.isArray(val)) {
      for (const v of val) {
        result.push(encode(key) + eq + encode(String(v)));
      }
    } else {
      result.push(encode(key) + eq + encode(String(val)));
    }
  }
  return result.join(sep);
}

function parse(str, sep, eq, options) {
  if (!str) return {};
  sep = sep || '&';
  eq = eq || '=';
  const decode = (options && options.decodeURIComponent) || decodeURIComponent;
  const maxKeys = (options && options.maxKeys) || 1000;
  const result = {};
  const pairs = str.split(sep).slice(0, maxKeys);
  for (const pair of pairs) {
    if (!pair) continue;
    const idx = pair.indexOf(eq);
    let key, val;
    if (idx >= 0) {
      key = decode(pair.slice(0, idx));
      val = decode(pair.slice(idx + 1));
    } else {
      key = decode(pair);
      val = '';
    }
    if (result.hasOwnProperty(key)) {
      if (!Array.isArray(result[key])) result[key] = [result[key]];
      result[key].push(val);
    } else {
      result[key] = val;
    }
  }
  return result;
}

function escape(str) {
  return encodeURIComponent(str);
}

function unescape(str) {
  return decodeURIComponent(str);
}

const querystring = {
  stringify,
  parse,
  encode: stringify,
  decode: parse,
  escape,
  unescape,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = querystring;
}
