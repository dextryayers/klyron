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
    } else if (val !== null && val !== undefined && typeof val !== 'function') {
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
  const maxKeys = (options && options.maxKeys !== undefined) ? options.maxKeys : 1000;
  const result = Object.create(null);
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
    if (Object.prototype.hasOwnProperty.call(result, key)) {
      if (!Array.isArray(result[key])) result[key] = [result[key]];
      result[key].push(val);
    } else {
      result[key] = val;
    }
  }
  return result;
}

function escape(str) {
  return encodeURIComponent(typeof str === 'string' ? str : String(str))
    .replace(/[!'()*]/g, function(c) {
      return '%' + c.charCodeAt(0).toString(16).toUpperCase();
    });
}

function unescape(str) {
  return decodeURIComponent((typeof str === 'string' ? str : String(str)).replace(/\+/g, ' '));
}

const querystring = {
  stringify,
  parse,
  encode: stringify,
  decode: parse,
  escape,
  unescape,
  formats: {
    RFC1738: 'RFC1738',
    RFC3986: 'RFC3986',
    default: 'RFC3986',
  },
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = querystring;
}
