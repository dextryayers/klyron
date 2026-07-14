// Klyron Runtime — node:https polyfill
// Wraps node:http with TLS defaults

const http = (typeof require === 'function' ? (function() { try { return require('./node_http'); } catch {} })() : null) || {};

const https = {
  ...http,
  createServer: (options, requestListener) => {
    if (typeof options === 'function') { requestListener = options; options = {}; }
    const server = http.createServer(requestListener);
    server._tlsOptions = options || {};
    return server;
  },
  request: (url, options, callback) => {
    if (typeof url === 'object' && !options) {
      if (typeof options === 'function') { callback = options; }
      options = url;
      url = options.href || `https://${options.hostname || options.host || 'localhost'}${options.path || options.pathname || '/'}`;
    }
    if (typeof options === 'function') { callback = options; options = {}; }
    if (!options) options = {};
    if (!url && options.href) url = options.href;
    if (!url && options.hostname) {
      url = `https://${options.hostname}${options.port ? ':' + options.port : ''}${options.path || options.pathname || '/'}`;
    }
    const req = http.request(url, { ...options, _noEnd: true }, callback);
    req.end();
    return req;
  },
  get: (url, options, callback) => {
    if (typeof options === 'function') { callback = options; options = {}; }
    return https.request(url, { ...options, method: 'GET' }, callback);
  },
  Agent: class Agent {
    constructor() { this.maxSockets = Infinity; this.sockets = {}; this.requests = {}; }
    destroy() {}
  },
  globalAgent: {},
  Server: function() {},
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = https;
}
