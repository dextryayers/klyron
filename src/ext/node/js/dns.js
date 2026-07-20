import { op_dns_lookup, op_dns_resolve, op_dns_reverse } from "ext:core/ops";

function normalizeOptions(options) {
  if (typeof options === "number") return { family: options };
  if (typeof options === "object" && options) return options;
  return {};
}

export function lookup(hostname, options, callback) {
  if (typeof options === "function") { callback = options; options = {}; }
  const opts = normalizeOptions(options);
  const family = opts.family || 0;
  const all = opts.all || false;

  let result;
  try {
    result = JSON.parse(op_dns_lookup(hostname, family));
  } catch (e) {
    if (callback) { callback(e); return; }
    throw e;
  }

  if (callback) {
    if (result.error) { callback(new Error(result.error)); return; }
    if (all) {
      callback(null, result.addresses);
    } else if (result.addresses && result.addresses.length > 0) {
      const addr = result.addresses[0];
      callback(null, addr.address, addr.family);
    } else {
      callback(null, null, 0);
    }
  }
  return result;
}

lookup.__promisify__ = lookup_promise;
export function lookup_promise(hostname, options) {
  return new Promise((resolve, reject) => {
    lookup(hostname, options, (err, ...args) => {
      if (err) reject(err);
      else resolve(args.length > 1 ? args : args[0]);
    });
  });
}

export function resolve(hostname, rrtype, callback) {
  if (typeof rrtype === "function") { callback = rrtype; rrtype = "A"; }
  if (typeof callback !== "function") throw new Error("Callback required");

  try {
    const result = JSON.parse(op_dns_resolve(hostname, rrtype.toUpperCase()));
    if (result.error) { callback(new Error(result.error)); return; }
    callback(null, result.entries);
  } catch (e) {
    callback(e);
  }
}

export function resolve4(hostname, options, callback) {
  if (typeof options === "function") { callback = options; options = {}; }
  const opts = normalizeOptions(options);
  lookup(hostname, { family: 4, all: true, ...opts }, (err, addrs) => {
    if (err) { callback?.(err); return; }
    const ips = (addrs || []).map(a => a.address);
    if (opts.ttl) {
      callback(null, ips.map(ip => ({ address: ip, ttl: 300 })));
    } else {
      callback(null, ips);
    }
  });
}

export function resolve6(hostname, options, callback) {
  if (typeof options === "function") { callback = options; options = {}; }
  const opts = normalizeOptions(options);
  lookup(hostname, { family: 6, all: true, ...opts }, (err, addrs) => {
    if (err) { callback?.(err); return; }
    const ips = (addrs || []).map(a => a.address);
    if (opts.ttl) {
      callback(null, ips.map(ip => ({ address: ip, ttl: 300 })));
    } else {
      callback(null, ips);
    }
  });
}

export function reverse(ip, callback) {
  try {
    const result = JSON.parse(op_dns_reverse(ip));
    if (result.error) { callback(new Error(result.error)); return; }
    callback(null, result.hostnames);
  } catch (e) {
    callback(e);
  }
}

export function resolveMx(hostname, callback) {
  resolve(hostname, "MX", callback);
}

export function resolveTxt(hostname, callback) {
  resolve(hostname, "TXT", callback);
}

export function resolveSrv(hostname, callback) {
  resolve(hostname, "SRV", callback);
}

export function resolveCname(hostname, callback) {
  resolve(hostname, "CNAME", callback);
}

export function resolveNs(hostname, callback) {
  resolve(hostname, "NS", callback);
}

// Promises API
export const promises = {
  lookup: lookup_promise,
  resolve4: (hostname, options) => new Promise((resolve, reject) => {
    resolve4(hostname, options, (err, r) => err ? reject(err) : resolve(r));
  }),
  resolve6: (hostname, options) => new Promise((resolve, reject) => {
    resolve6(hostname, options, (err, r) => err ? reject(err) : resolve(r));
  }),
  resolve: (hostname, rrtype) => new Promise((resolve, reject) => {
    resolve(hostname, rrtype, (err, r) => err ? reject(err) : resolve(r));
  }),
  reverse: (ip) => new Promise((resolve, reject) => {
    reverse(ip, (err, r) => err ? reject(err) : resolve(r));
  }),
};

export default {
  lookup, resolve, resolve4, resolve6, reverse,
  resolveMx, resolveTxt, resolveSrv, resolveCname, resolveNs,
  promises,
};
