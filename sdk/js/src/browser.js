class KlyronRuntime {
  constructor(options = {}) {
    this._options = options;
  }

  get fs() {
    throw new Error('FS operations not available in browser');
  }

  get process() {
    return {
      pid: -1,
      cwd: '/',
      platform: navigator?.platform || 'unknown',
    };
  }

  get env() {
    return {
      get: () => null,
      set: () => {},
      getAll: () => ({}),
      has: () => false,
    };
  }

  get http() {
    const _fetch = globalThis.fetch.bind(globalThis);
    return {
      get: (url, headers) => _fetch(url, { headers }),
      post: (url, body, headers) => _fetch(url, { method: 'POST', headers, body }),
      put: (url, body, headers) => _fetch(url, { method: 'PUT', headers, body }),
      del: (url, headers) => _fetch(url, { method: 'DELETE', headers }),
      request: (method, url, opts) => _fetch(url, { method, ...opts }),
    };
  }

  async version() {
    return '0.1.0';
  }

  async eval(code, lang = 'js') {
    throw new Error('eval not available in browser');
  }
}

export default KlyronRuntime;
