import { op_zlib_gzip, op_zlib_gunzip, op_zlib_deflate, op_zlib_inflate } from "ext:core/ops";
import { Buffer } from "./buffer.js";

function toBuf(data) {
  return Buffer.isBuffer(data) ? data : Buffer.from(String(data), "utf8");
}

function toNodeBuf(raw) {
  return Buffer.from(raw);
}

function wrapCallback(block, callback) {
  try {
    const result = block();
    if (callback) callback(null, toNodeBuf(result));
    return toNodeBuf(result);
  } catch (e) {
    if (callback) callback(e);
    throw e;
  }
}

export function gzip(buf, callback) {
  return wrapCallback(() => op_zlib_gzip(toBuf(buf)), callback);
}
export function gzipSync(buf) {
  return toNodeBuf(op_zlib_gzip(toBuf(buf)));
}

export function gunzip(buf, callback) {
  return wrapCallback(() => op_zlib_gunzip(toBuf(buf)), callback);
}
export function gunzipSync(buf) {
  return toNodeBuf(op_zlib_gunzip(toBuf(buf)));
}

export function deflate(buf, callback) {
  return wrapCallback(() => op_zlib_deflate(toBuf(buf)), callback);
}
export function deflateSync(buf) {
  return toNodeBuf(op_zlib_deflate(toBuf(buf)));
}

export function inflate(buf, callback) {
  return wrapCallback(() => op_zlib_inflate(toBuf(buf)), callback);
}
export function inflateSync(buf) {
  return toNodeBuf(op_zlib_inflate(toBuf(buf)));
}

export function deflateRaw(buf, callback) {
  return deflate(buf, callback);
}
export function deflateRawSync(buf) {
  return deflateSync(buf);
}

export function inflateRaw(buf, callback) {
  return inflate(buf, callback);
}
export function inflateRawSync(buf) {
  return inflateSync(buf);
}

function _promisify(fn) {
  return (buf) => new Promise((resolve, reject) => {
    try { resolve(fn(buf)); } catch (e) { reject(e); }
  });
}

export const promises = {
  gzip: _promisify(gzipSync),
  gunzip: _promisify(gunzipSync),
  deflate: _promisify(deflateSync),
  inflate: _promisify(inflateSync),
  deflateRaw: _promisify(deflateRawSync),
  inflateRaw: _promisify(inflateRawSync),
};

export default {
  gzip, gzipSync, gunzip, gunzipSync,
  deflate, deflateSync, inflate, inflateSync,
  deflateRaw, deflateRawSync, inflateRaw, inflateRawSync,
  promises,
  constants: {
    Z_OK: 0, Z_STREAM_END: 1, Z_NEED_DICT: 2,
    Z_ERRNO: -1, Z_STREAM_ERROR: -2, Z_DATA_ERROR: -3,
    Z_MEM_ERROR: -4, Z_BUF_ERROR: -5, Z_VERSION_ERROR: -6,
    Z_NO_FLUSH: 0, Z_PARTIAL_FLUSH: 1, Z_SYNC_FLUSH: 2,
    Z_FULL_FLUSH: 3, Z_FINISH: 4, Z_BLOCK: 5, Z_TREES: 6,
    Z_DEFLATED: 8,
  },
};
