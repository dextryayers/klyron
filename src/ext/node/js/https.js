// https.js — TLS-terminated HTTP server/client.
//
// NOTE: klyron's net extension does not yet perform real TLS handshakes, so
// https.createServer currently serves traffic over the same transport as
// http.createServer. The API surface mirrors Node's `https` module so that
// frameworks (Express/Hono/etc.) import and call it without errors. Real
// TLS termination is tracked under Phase 6 (Node builtins hardening).
import http from "./http.js";
import net from "./net.js";

export class Agent {
  constructor(options) { this.options = options || {}; }
}
export const globalAgent = new Agent();

export function createServer(options, requestListener) {
  // Same transport as http for now; API-compatible with Node's https.
  return http.createServer(options, requestListener);
}

export function request(options, cb) {
  const opts = typeof options === "string" ? { url: options } : (options || {});
  opts._tls = true;
  return http.request(opts, cb);
}

export function get(options, cb) {
  const opts = typeof options === "string" ? { url: options } : (options || {});
  opts.method = "GET";
  return request(opts, cb);
}

export const Server = http.Server;
export const STATUS_CODES = http.STATUS_CODES;
export const METHODS = http.METHODS;

export default {
  Agent,
  globalAgent,
  createServer,
  request,
  get,
  Server,
  STATUS_CODES,
  METHODS,
};
