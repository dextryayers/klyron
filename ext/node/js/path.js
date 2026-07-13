const sep = "/";
const delimiter = ":";

export { sep, delimiter };

export function basename(p, ext) {
  p = String(p);
  const last = p.lastIndexOf(sep);
  let base = last >= 0 ? p.slice(last + 1) : p;
  if (ext && base.endsWith(ext)) base = base.slice(0, -ext.length);
  return base || "/";
}

export function dirname(p) {
  p = String(p);
  if (p === "/") return "/";
  const last = p.lastIndexOf(sep);
  if (last <= 0) return p[0] === sep ? "/" : ".";
  return p.slice(0, last) || "/";
}

export function extname(p) {
  p = String(p);
  const base = basename(p);
  const dot = base.lastIndexOf(".");
  if (dot <= 0) return "";
  return base.slice(dot);
}

export function join(...parts) {
  return normalize(parts.filter(p => p != null).join(sep));
}

export function resolve(...parts) {
  let resolved = "";
  for (const p of parts) {
    if (p && p[0] === sep) resolved = p;
    else if (resolved) resolved = resolved + sep + p;
    else resolved = p || ".";
  }
  return normalize(resolved || ".");
}

export function normalize(p) {
  p = String(p);
  if (!p) return ".";
  const leading = p[0] === sep;
  const trailing = p.length > 1 && p[p.length - 1] === sep;
  const parts = p.split(sep).filter(Boolean);
  const out = [];
  for (const part of parts) {
    if (part === ".") continue;
    if (part === "..") { if (out.length && out[out.length - 1] !== "..") out.pop(); else out.push(".."); }
    else out.push(part);
  }
  let result = leading ? sep + out.join(sep) : out.join(sep) || ".";
  if (trailing && out.length) result += sep;
  return result;
}

export function relative(from, to) {
  from = resolve(from).split(sep).filter(Boolean);
  to = resolve(to).split(sep).filter(Boolean);
  let i = 0;
  while (i < from.length && i < to.length && from[i] === to[i]) i++;
  const up = from.slice(i).map(() => "..");
  return up.concat(to.slice(i)).join(sep) || ".";
}

export function isAbsolute(p) { return String(p)[0] === sep; }

export function parse(p) {
  const root = p[0] === sep ? sep : "";
  const base = basename(p);
  return { root, dir: dirname(p), base, ext: extname(base), name: base.slice(0, -extname(base).length || undefined) };
}

export function format(obj) {
  const base = obj.base || (obj.name || "") + (obj.ext || "");
  if (!obj.dir) return base;
  return join(obj.dir, base);
}

export default { sep, delimiter, basename, dirname, extname, join, resolve, normalize, relative, isAbsolute, parse, format };
