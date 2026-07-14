export function parse(str, sep, eq, options) {
  sep = sep || "&";
  eq = eq || "=";
  const result = {};
  if (!str) return result;
  str.split(sep).forEach(pair => {
    if (!pair) return;
    const [k, ...v] = pair.split(eq);
    const key = decodeURIComponent(k.replace(/\+/g, " "));
    const val = decodeURIComponent(v.join(eq).replace(/\+/g, " "));
    if (options?.maxKeys && Object.keys(result).length >= options.maxKeys) return;
    if (result[key] !== undefined) {
      if (!Array.isArray(result[key])) result[key] = [result[key]];
      result[key].push(val);
    } else result[key] = val;
  });
  return result;
}

export function stringify(obj, sep, eq, options) {
  sep = sep || "&";
  eq = eq || "=";
  const pairs = [];
  for (const k of Object.keys(obj || {})) {
    const v = obj[k];
    if (Array.isArray(v)) { v.forEach(x => pairs.push(enc(k) + eq + enc(x))); }
    else pairs.push(enc(k) + eq + enc(v));
  }
  return pairs.join(sep);
}

function enc(s) { return encodeURIComponent(String(s)).replace(/%20/g, "+"); }

export function decode(str) { return parse(str); }
export function encode(obj) { return stringify(obj); }

export default { parse, stringify, decode, encode };
