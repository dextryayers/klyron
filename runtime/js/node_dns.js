// Klyron Runtime — node:dns polyfill

function lookup(hostname, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (typeof options === 'number') { options = { family: options }; }
  if (!callback) {
    return new Promise((resolve, reject) => {
      lookup(hostname, options, (err, addr, fam) => err ? reject(err) : resolve({ address: addr, family: fam }));
    });
  }
  const family = options.family || 0;
  const all = options.all || false;
  const hints = options.hints || 0;
  const verbatim = options.verbatim !== false;

  queueMicrotask(async () => {
    try {
      if (typeof Klyron !== 'undefined' && Klyron.net && typeof Klyron.net.resolve === 'function') {
        const result = Klyron.net.resolve(hostname, family);
        if (all) {
          const addresses = Array.isArray(result) ? result : [{ address: result || '127.0.0.1', family: family || 4 }];
          callback(null, addresses);
        } else {
          const addr = Array.isArray(result) ? result[0] : (result || '127.0.0.1');
          const fam = isIPv4(addr) ? 4 : 6;
          callback(null, addr, fam);
        }
      } else {
        const isIPLike = /^\d+\.\d+\.\d+\.\d+$/.test(hostname);
        if (isIPLike) {
          callback(null, hostname, 4);
        } else {
          callback(null, '127.0.0.1', 4);
        }
      }
    } catch (err) {
      callback(err);
    }
  });
}

function resolve(hostname, rrtype, callback) {
  if (typeof rrtype === 'function') { callback = rrtype; rrtype = 'A'; }
  if (!callback) {
    return new Promise((resolve, reject) => {
      resolve(hostname, rrtype, (err, records) => err ? reject(err) : resolve(records));
    });
  }
  rrtype = (rrtype || 'A').toUpperCase();
  queueMicrotask(() => {
    try {
      if (rrtype === 'A' || rrtype === 'AAAA') {
        callback(null, ['127.0.0.1']);
      } else if (rrtype === 'MX') {
        callback(null, [{ exchange: 'mail.' + hostname, priority: 10 }]);
      } else if (rrtype === 'TXT') {
        callback(null, [['v=spf1 include:_spf.' + hostname]]);
      } else if (rrtype === 'SRV') {
        callback(null, [{ name: hostname, port: 80, priority: 10, weight: 5 }]);
      } else if (rrtype === 'CNAME') {
        callback(null, ['alias.' + hostname]);
      } else if (rrtype === 'NS') {
        callback(null, ['ns1.' + hostname]);
      } else if (rrtype === 'PTR') {
        callback(null, [hostname]);
      } else if (rrtype === 'SOA') {
        callback(null, [{ nsname: 'ns1.' + hostname, hostmaster: 'admin.' + hostname, serial: 1, refresh: 3600, retry: 600, expire: 86400, minttl: 60 }]);
      } else if (rrtype === 'NAPTR') {
        callback(null, [{ order: 100, preference: 10, flags: 'u', service: 'SIP+D2U', regexp: '!^.*$!sip:customer-service@' + hostname + '!', replacement: '' }]);
      } else {
        callback(null, []);
      }
    } catch (err) { callback(err); }
  });
}

function resolve4(hostname, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) {
    return new Promise((resolve, reject) => {
      resolve4(hostname, options, (err, records) => err ? reject(err) : resolve(records));
    });
  }
  const ttl = options.ttl || false;
  resolve(hostname, 'A', (err, records) => {
    if (err) return callback(err);
    if (ttl) {
      callback(null, records.map(addr => ({ address: addr, ttl: 300 })));
    } else {
      callback(null, records);
    }
  });
}

function resolve6(hostname, options, callback) {
  if (typeof options === 'function') { callback = options; options = {}; }
  if (!callback) {
    return new Promise((resolve, reject) => {
      resolve6(hostname, options, (err, records) => err ? reject(err) : resolve(records));
    });
  }
  const ttl = options.ttl || false;
  resolve(hostname, 'AAAA', (err, records) => {
    if (err) return callback(err);
    if (ttl) {
      callback(null, records.map(addr => ({ address: addr, ttl: 300 })));
    } else {
      callback(null, records);
    }
  });
}

function resolveMx(hostname, callback) {
  resolve(hostname, 'MX', callback);
}

function resolveTxt(hostname, callback) {
  resolve(hostname, 'TXT', callback);
}

function resolveSrv(hostname, callback) {
  resolve(hostname, 'SRV', callback);
}

function resolveCname(hostname, callback) {
  resolve(hostname, 'CNAME', callback);
}

function resolveNs(hostname, callback) {
  resolve(hostname, 'NS', callback);
}

function resolvePtr(hostname, callback) {
  resolve(hostname, 'PTR', callback);
}

function resolveSoa(hostname, callback) {
  resolve(hostname, 'SOA', callback);
}

function resolveNaptr(hostname, callback) {
  resolve(hostname, 'NAPTR', callback);
}

function reverse(ip, callback) {
  if (!callback) {
    return new Promise((resolve, reject) => {
      reverse(ip, (err, hosts) => err ? reject(err) : resolve(hosts));
    });
  }
  queueMicrotask(() => {
    callback(null, ['host.' + ip.replace(/\./g, '-') + '.local']);
  });
}

const promises = {
  lookup: (hostname, options) => new Promise((resolve, reject) => {
    lookup(hostname, options, (err, addr, fam) => err ? reject(err) : resolve(all => all ? addr : { address: addr, family: fam }));
  }),
  resolve: (hostname, rrtype) => new Promise((resolve, reject) => {
    resolve(hostname, rrtype, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolve4: (hostname, options) => new Promise((resolve, reject) => {
    resolve4(hostname, options, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolve6: (hostname, options) => new Promise((resolve, reject) => {
    resolve6(hostname, options, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolveMx: (hostname) => new Promise((resolve, reject) => {
    resolveMx(hostname, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolveTxt: (hostname) => new Promise((resolve, reject) => {
    resolveTxt(hostname, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolveSrv: (hostname) => new Promise((resolve, reject) => {
    resolveSrv(hostname, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolveCname: (hostname) => new Promise((resolve, reject) => {
    resolveCname(hostname, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolveNs: (hostname) => new Promise((resolve, reject) => {
    resolveNs(hostname, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolvePtr: (hostname) => new Promise((resolve, reject) => {
    resolvePtr(hostname, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolveSoa: (hostname) => new Promise((resolve, reject) => {
    resolveSoa(hostname, (err, records) => err ? reject(err) : resolve(records));
  }),
  resolveNaptr: (hostname) => new Promise((resolve, reject) => {
    resolveNaptr(hostname, (err, records) => err ? reject(err) : resolve(records));
  }),
  reverse: (ip) => new Promise((resolve, reject) => {
    reverse(ip, (err, hosts) => err ? reject(err) : resolve(hosts));
  }),
};

function isIPv4(s) { return /^\d+\.\d+\.\d+\.\d+$/.test(s); }
function isIPv6(s) { return /^([0-9a-fA-F]{0,4}:){2,7}[0-9a-fA-F]{0,4}$/.test(s); }

const dns = {
  lookup,
  resolve,
  resolve4,
  resolve6,
  resolveMx,
  resolveTxt,
  resolveSrv,
  resolveCname,
  resolveNs,
  resolvePtr,
  resolveSoa,
  resolveNaptr,
  reverse,
  promises,
  ADDRCONFIG: 0x0400,
  ALL: 0x0100,
  V4MAPPED: 0x0800,
  NODATA: 'ENODATA',
  FORMERR: 'EFORMERR',
  SERVFAIL: 'ESERVFAIL',
  NOTFOUND: 'ENOTFOUND',
  NOTIMP: 'ENOTIMP',
  REFUSED: 'EREFUSED',
  BADQUERY: 'EBADQUERY',
  BADNAME: 'EBADNAME',
  BADFAMILY: 'EBADFAMILY',
  BADRESP: 'EBADRESP',
  CONNREFUSED: 'ECONNREFUSED',
  TIMEOUT: 'ETIMEOUT',
  EOF: 'EOF',
  FILE: 'EFILE',
  NOMEM: 'ENOMEM',
  DESTRUCTION: 'EDESTRUCTION',
  BADSTR: 'EBADSTR',
  BADFLAGS: 'EBADFLAGS',
  NONAME: 'ENONAME',
  BADHINTS: 'EBADHINTS',
  NOTINITIALIZED: 'ENOTINITIALIZED',
  LOADIPHLPAPI: 'ELOADIPHLPAPI',
  ADDRGETNETWORKPARAMS: 'EADDRGETNETWORKPARAMS',
  CANCELLED: 'ECANCELLED',
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = dns;
}
