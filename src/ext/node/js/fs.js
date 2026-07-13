import { op_fs_read_file, op_fs_write_file, op_fs_mkdir, op_fs_read_dir, op_fs_stat, op_fs_exists, op_fs_remove, op_fs_copy, op_fs_rename } from "ext:core/ops";
import * as path from "./path.js";

export function readFileSync(p, encoding) {
  const data = op_fs_read_file(typeof p === "string" ? p : p.path || p.toString());
  if (encoding === "utf8" || encoding === "utf-8" || !encoding) return data;
  return Buffer.from(data);
}

export function writeFileSync(p, data, encoding) {
  const str = typeof data === "string" ? data : (data?.toString?.() || String(data));
  return op_fs_write_file(typeof p === "string" ? p : p.path || p.toString(), str);
}

export function existsSync(p) { return op_fs_exists(typeof p === "string" ? p : p.path || p.toString()); }

export function mkdirSync(p, opts) {
  if (opts?.recursive) return op_fs_mkdir(String(p));
  return op_fs_mkdir(String(p));
}

export function readdirSync(p, opts) {
  const entries = op_fs_read_dir(String(p));
  if (opts?.withFileTypes) return entries.map(e => ({ name: e.name, isFile: () => e.is_file, isDirectory: () => e.is_dir, isSymbolicLink: () => e.is_symlink }));
  return entries.map(e => e.name);
}

export function statSync(p) {
  const s = op_fs_stat(String(p));
  return {
    isFile: () => s.is_file,
    isDirectory: () => s.is_dir,
    isSymbolicLink: () => s.is_symlink,
    size: s.size,
    birthtimeMs: s.created * 1000,
    mtimeMs: s.modified * 1000,
    atimeMs: s.accessed * 1000,
    ctimeMs: s.modified * 1000,
  };
}

export function lstatSync(p) { return statSync(p); }

export function unlinkSync(p) { return op_fs_remove(String(p)); }

export function rmdirSync(p, opts) {
  if (opts?.recursive) return op_fs_remove(String(p));
  return op_fs_remove(String(p));
}

export function copyFileSync(src, dest) { return op_fs_copy(String(src), String(dest)); }

export function renameSync(oldPath, newPath) { return op_fs_rename(String(oldPath), String(newPath)); }

export function appendFileSync(p, data) {
  const existing = existsSync(p) ? readFileSync(p, "utf8") : "";
  writeFileSync(p, existing + (typeof data === "string" ? data : data.toString()));
}

export function readdir(p, opts, cb) {
  if (typeof opts === "function") { cb = opts; opts = {}; }
  try { const r = readdirSync(p, opts); cb?.(null, r); return Promise.resolve(r); }
  catch (e) { cb?.(e); return Promise.reject(e); }
}

export function readFile(p, encoding, cb) {
  if (typeof encoding === "function") { cb = encoding; encoding = "utf8"; }
  try { const r = readFileSync(p, encoding); cb?.(null, r); return Promise.resolve(r); }
  catch (e) { cb?.(e); return Promise.reject(e); }
}

export function writeFile(p, data, encoding, cb) {
  if (typeof encoding === "function") { cb = encoding; encoding = "utf8"; }
  try { writeFileSync(p, data, encoding); cb?.(); return Promise.resolve(); }
  catch (e) { cb?.(e); return Promise.reject(e); }
}

export function mkdir(p, opts, cb) {
  if (typeof opts === "function") { cb = opts; opts = {}; }
  try { mkdirSync(p, opts); cb?.(); return Promise.resolve(); }
  catch (e) { cb?.(e); return Promise.reject(e); }
}

export function stat(p, cb) {
  try { const r = statSync(p); cb?.(null, r); return Promise.resolve(r); }
  catch (e) { cb?.(e); return Promise.reject(e); }
}

export function exists(p, cb) {
  try { const r = existsSync(p); cb?.(r); return Promise.resolve(r); }
  catch (e) { return Promise.resolve(false); }
}

export const promises = { readFile, writeFile, mkdir, readdir, stat, readdir: readdir, unlink: unlinkSync, rmdir: rmdirSync, copyFile: copyFileSync, rename: renameSync, appendFile: appendFileSync };

export const constants = { F_OK: 0, R_OK: 4, W_OK: 2, X_OK: 1 };

export default { readFileSync, writeFileSync, existsSync, mkdirSync, readdirSync, statSync, lstatSync, unlinkSync, rmdirSync, copyFileSync, renameSync, appendFileSync, readFile, writeFile, mkdir, readdir, stat, exists, promises, constants };
