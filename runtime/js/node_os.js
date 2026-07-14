// Klyron Runtime — node:os polyfill
// OS information, constants, utilities

const EOL = '\n';

function arch() {
  if (typeof process !== 'undefined' && process.arch) return process.arch;
  return 'x64';
}

function platform() {
  if (typeof process !== 'undefined' && process.platform) return process.platform;
  return 'linux';
}

function type() {
  if (typeof navigator !== 'undefined' && navigator.userAgent) {
    if (navigator.userAgent.includes('Linux')) return 'Linux';
    if (navigator.userAgent.includes('Mac')) return 'Darwin';
    if (navigator.userAgent.includes('Win')) return 'Windows_NT';
  }
  return 'Linux';
}

function release() {
  return '6.8.0';
}

function hostname() {
  if (typeof process !== 'undefined' && process.env && process.env.HOSTNAME) return process.env.HOSTNAME;
  if (typeof process !== 'undefined' && process.env && process.env.COMPUTERNAME) return process.env.COMPUTERNAME;
  return 'localhost';
}

function homedir() {
  if (typeof process !== 'undefined' && process.env) {
    if (process.env.HOME) return process.env.HOME;
    if (process.env.USERPROFILE) return process.env.USERPROFILE;
    if (process.env.HOMEDRIVE && process.env.HOMEPATH) return process.env.HOMEDRIVE + process.env.HOMEPATH;
  }
  return '/home/user';
}

function tmpdir() {
  if (typeof process !== 'undefined' && process.env) {
    if (process.env.TMPDIR) return process.env.TMPDIR;
    if (process.env.TMP) return process.env.TMP;
    if (process.env.TEMP) return process.env.TEMP;
  }
  return '/tmp';
}

function cpus() {
  const cpus = [];
  const count = typeof navigator !== 'undefined' && navigator.hardwareConcurrency ? navigator.hardwareConcurrency : 4;
  for (let i = 0; i < count; i++) {
    cpus.push({
      model: 'Klyron Virtual CPU',
      speed: 2400,
      times: { user: 0, nice: 0, sys: 0, idle: 0, irq: 0 },
    });
  }
  return cpus;
}

function totalmem() {
  return 8 * 1024 * 1024 * 1024;
}

function freemem() {
  return 4 * 1024 * 1024 * 1024;
}

function uptime() {
  return process.uptime ? process.uptime() : 0;
}

function loadavg() {
  return [0, 0, 0];
}

function networkInterfaces() {
  return {
    lo: [{ address: '127.0.0.1', netmask: '255.0.0.0', family: 'IPv4', mac: '00:00:00:00:00:00', internal: true, cidr: '127.0.0.1/8' }],
  };
}

function userInfo(options) {
  const encoding = (options && options.encoding) || 'utf8';
  const username = typeof process !== 'undefined' && process.env ? process.env.USER || process.env.USERNAME || 'root' : 'root';
  const result = {
    username,
    uid: 1000,
    gid: 1000,
    shell: '/bin/bash',
    homedir: homedir(),
  };
  if (encoding === 'buffer') {
    for (const key of Object.keys(result)) {
      if (typeof result[key] === 'string') result[key] = Buffer.from(result[key]);
    }
  }
  return result;
}

function endianness() {
  const buf = new ArrayBuffer(2);
  const view = new DataView(buf);
  view.setInt16(0, 0x0102, false);
  return view.getInt8(0) === 0x01 ? 'BE' : 'LE';
}

function freememPercentage() {
  return freemem() / totalmem();
}

function availableParallelism() {
  return typeof navigator !== 'undefined' && navigator.hardwareConcurrency ? navigator.hardwareConcurrency : 4;
}

function getPriority(pid) { return 0; }
function setPriority(pid, priority) {}

function version() {
  return '22.14.0';
}

const os = {
  EOL,
  arch,
  platform,
  type,
  release,
  hostname,
  homedir,
  tmpdir,
  cpus,
  totalmem,
  freemem,
  uptime,
  loadavg,
  networkInterfaces,
  userInfo,
  endianness,
  availableParallelism,
  getPriority,
  setPriority,
  version,
  constants: {
    UV_UDP_REUSEADDR: 4,
    dlopen: {},
    errno: {
      E2BIG: 7, EACCES: 13, EADDRINUSE: 98, EADDRNOTAVAIL: 99,
      EAFNOSUPPORT: 97, EAGAIN: 11, EALREADY: 114, EBADF: 9,
      EBADMSG: 74, EBUSY: 16, ECANCELED: 125, ECHILD: 10,
      ECONNABORTED: 103, ECONNREFUSED: 111, ECONNRESET: 104,
      EDEADLK: 35, EDESTADDRREQ: 89, EDOM: 33, EDQUOT: 122,
      EEXIST: 17, EFAULT: 14, EFBIG: 27, EHOSTUNREACH: 113,
      EIDRM: 43, EILSEQ: 84, EINPROGRESS: 115, EINTR: 4,
      EINVAL: 22, EIO: 5, EISCONN: 106, EISDIR: 21,
      ELOOP: 40, EMFILE: 24, EMLINK: 31, EMSGSIZE: 90,
      EMULTIHOP: 72, ENAMETOOLONG: 36, ENETDOWN: 100,
      ENETRESET: 102, ENETUNREACH: 101, ENFILE: 23,
      ENOBUFS: 105, ENODATA: 61, ENODEV: 19, ENOENT: 2,
      ENOEXEC: 8, ENOLCK: 37, ENOLINK: 67, ENOMEM: 12,
      ENOMSG: 42, ENOPROTOOPT: 92, ENOSPC: 28, ENOSR: 63,
      ENOSTR: 60, ENOSYS: 38, ENOTCONN: 107, ENOTDIR: 20,
      ENOTEMPTY: 39, ENOTSOCK: 88, ENOTSUP: 95, ENOTTY: 25,
      ENXIO: 6, EOPNOTSUPP: 95, EOVERFLOW: 75, EPERM: 1,
      EPIPE: 32, EPROTO: 71, EPROTONOSUPPORT: 93,
      EPROTOTYPE: 91, ERANGE: 34, EROFS: 30, ESPIPE: 29,
      ESRCH: 3, ESTALE: 70, ETIME: 62, ETIMEDOUT: 110,
      ETXTBSY: 26, EWOULDBLOCK: 11, EXDEV: 18,
    },
    signals: {
      SIGHUP: 1, SIGINT: 2, SIGQUIT: 3, SIGILL: 4, SIGTRAP: 5,
      SIGABRT: 6, SIGIOT: 6, SIGBUS: 7, SIGFPE: 8, SIGKILL: 9,
      SIGUSR1: 10, SIGSEGV: 11, SIGUSR2: 12, SIGPIPE: 13,
      SIGALRM: 14, SIGTERM: 15, SIGSTKFLT: 16, SIGCHLD: 17,
      SIGCONT: 18, SIGSTOP: 19, SIGTSTP: 20, SIGTTIN: 21,
      SIGTTOU: 22, SIGURG: 23, SIGXCPU: 24, SIGXFSZ: 25,
      SIGVTALRM: 26, SIGPROF: 27, SIGWINCH: 28, SIGIO: 29,
      SIGPWR: 30, SIGSYS: 31,
    },
    priority: {
      PRIORITY_LOW: 19, PRIORITY_BELOW_NORMAL: 10,
      PRIORITY_NORMAL: 0, PRIORITY_ABOVE_NORMAL: -7,
      PRIORITY_HIGH: -14, PRIORITY_HIGHEST: -20,
    },
  },
  devNull: '/dev/null',
  machine: () => 'x86_64',
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = os;
}
