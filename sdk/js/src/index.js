class KlyronRuntime {
  constructor(options = {}) {
    this._options = options;
    this._fs = null;
    this._http = null;
    this._process = null;
    this._env = null;
  }

  get fs() {
    if (!this._fs) {
      this._fs = new KlyronFS(this);
    }
    return this._fs;
  }

  get http() {
    if (!this._http) {
      this._http = new KlyronHTTP(this);
    }
    return this._http;
  }

  get process() {
    if (!this._process) {
      this._process = new KlyronProcess(this);
    }
    return this._process;
  }

  get env() {
    if (!this._env) {
      this._env = new KlyronEnv(this);
    }
    return this._env;
  }

  async version() {
    return '0.1.0';
  }

  async eval(code, lang = 'js') {
    const url = `klyron://eval/${lang}`;
    const response = await fetch(url, {
      method: 'POST',
      body: code,
    });
    return response.text();
  }

  async run(path, lang) {
    const source = await this.fs.read(path);
    return this.eval(source, lang || path.split('.').pop());
  }
}

class KlyronFS {
  constructor(runtime) {
    this._runtime = runtime;
  }

  async read(path) {
    if (typeof process !== 'undefined' && process.versions && process.versions.node) {
      const fs = await import('fs/promises');
      return fs.readFile(path, 'utf-8');
    }
    const response = await fetch(path);
    return response.text();
  }

  async write(path, content) {
    if (typeof process !== 'undefined' && process.versions && process.versions.node) {
      const fs = await import('fs/promises');
      return fs.writeFile(path, content, 'utf-8');
    }
    throw new Error('FS write not supported in browser');
  }

  async exists(path) {
    if (typeof process !== 'undefined' && process.versions && process.versions.node) {
      const fs = await import('fs/promises');
      return fs.access(path).then(() => true).catch(() => false);
    }
    return false;
  }

  async list(dir = '.') {
    if (typeof process !== 'undefined' && process.versions && process.versions.node) {
      const fs = await import('fs/promises');
      return fs.readdir(dir);
    }
    throw new Error('FS list not supported in browser');
  }

  async mkdir(dir) {
    if (typeof process !== 'undefined' && process.versions && process.versions.node) {
      const fs = await import('fs/promises');
      return fs.mkdir(dir, { recursive: true });
    }
    throw new Error('FS mkdir not supported in browser');
  }

  async remove(path) {
    if (typeof process !== 'undefined' && process.versions && process.versions.node) {
      const fs = await import('fs/promises');
      return fs.rm(path, { recursive: true, force: true });
    }
    throw new Error('FS remove not supported in browser');
  }
}

class KlyronHTTP {
  constructor(runtime) {
    this._runtime = runtime;
  }

  async get(url, headers = {}) {
    return fetch(url, { headers });
  }

  async post(url, body, headers = {}) {
    return fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...headers },
      body: typeof body === 'string' ? body : JSON.stringify(body),
    });
  }

  async put(url, body, headers = {}) {
    return fetch(url, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json', ...headers },
      body: typeof body === 'string' ? body : JSON.stringify(body),
    });
  }

  async del(url, headers = {}) {
    return fetch(url, { method: 'DELETE', headers });
  }

  async request(method, url, options = {}) {
    return fetch(url, { method, ...options });
  }
}

class KlyronProcess {
  constructor(runtime) {
    this._runtime = runtime;
  }

  async exec(command, args = []) {
    if (typeof process !== 'undefined' && process.versions && process.versions.node) {
      const { execFile } = await import('child_process');
      return new Promise((resolve, reject) => {
        const child = execFile(command, args, (error, stdout, stderr) => {
          if (error) reject(error);
          else resolve({ stdout, stderr, code: 0 });
        });
      });
    }
    throw new Error('Process exec not supported in browser');
  }

  async spawn(command, args = []) {
    if (typeof process !== 'undefined' && process.versions && process.versions.node) {
      const { spawn } = await import('child_process');
      const child = spawn(command, args);
      return child;
    }
    throw new Error('Process spawn not supported in browser');
  }

  get pid() {
    return typeof process !== 'undefined' ? process.pid : -1;
  }

  get cwd() {
    return typeof process !== 'undefined' ? process.cwd() : '/';
  }

  get platform() {
    return typeof process !== 'undefined' ? process.platform : 'unknown';
  }
}

class KlyronEnv {
  constructor(runtime) {
    this._runtime = runtime;
  }

  get(key) {
    if (typeof process !== 'undefined' && process.env) {
      return process.env[key] || null;
    }
    return null;
  }

  set(key, value) {
    if (typeof process !== 'undefined' && process.env) {
      process.env[key] = value;
    }
  }

  getAll() {
    if (typeof process !== 'undefined' && process.env) {
      return { ...process.env };
    }
    return {};
  }

  has(key) {
    return this.get(key) !== null;
  }
}

if (typeof module !== 'undefined' && module.exports) {
  module.exports = KlyronRuntime;
  module.exports.KlyronRuntime = KlyronRuntime;
  module.exports.KlyronFS = KlyronFS;
  module.exports.KlyronHTTP = KlyronHTTP;
  module.exports.KlyronProcess = KlyronProcess;
  module.exports.KlyronEnv = KlyronEnv;
}

export default KlyronRuntime;
export { KlyronRuntime, KlyronFS, KlyronHTTP, KlyronProcess, KlyronEnv };
