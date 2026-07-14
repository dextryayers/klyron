// Klyron Runtime — node:path polyfill

const CHAR_DOT = 46;
const CHAR_FORWARD_SLASH = 47;
const CHAR_BACKWARD_SLASH = 92;

function isPosixPathSeparator(code) {
  return code === CHAR_FORWARD_SLASH;
}

function isPathSeparator(code) {
  return isPosixPathSeparator(code) || code === CHAR_BACKWARD_SLASH;
}

function _format(sep, pathObject) {
  const dir = pathObject.dir || pathObject.root;
  const base = pathObject.base || (pathObject.name || '') + (pathObject.ext || '');
  if (!dir) return base;
  if (dir === pathObject.root) return dir + base;
  return dir + sep + base;
}

const posix = {
  sep: '/',
  delimiter: ':',

  resolve(...args) {
    let resolvedPath = '';
    let resolvedAbsolute = false;
    for (let i = args.length - 1; i >= -1 && !resolvedAbsolute; i--) {
      const path = i >= 0 ? args[i] : process.cwd();
      if (typeof path !== 'string') throw new TypeError('Path must be a string');
      if (path.length === 0) continue;
      resolvedPath = path + '/' + resolvedPath;
      resolvedAbsolute = path.charCodeAt(0) === CHAR_FORWARD_SLASH;
    }
    resolvedPath = normalizeString(resolvedPath, !resolvedAbsolute);
    if (resolvedAbsolute && resolvedPath.length > 0) return '/' + resolvedPath;
    if (resolvedAbsolute) return '/';
    return resolvedPath.length > 0 ? resolvedPath : '.';
  },

  normalize(path) {
    if (path.length === 0) return '.';
    const isAbsolute = path.charCodeAt(0) === CHAR_FORWARD_SLASH;
    const trailingSeparator = path.charCodeAt(path.length - 1) === CHAR_FORWARD_SLASH;
    path = normalizeString(path, !isAbsolute);
    if (path.length === 0 && !isAbsolute) path = '.';
    if (path.length > 0 && trailingSeparator) path += '/';
    if (isAbsolute) return '/' + path;
    return path;
  },

  join(...args) {
    if (args.length === 0) return '.';
    let joined = '';
    for (let i = 0; i < args.length; i++) {
      const arg = args[i];
      if (typeof arg !== 'string') throw new TypeError('Path must be a string');
      if (arg.length > 0) {
        if (joined.length === 0) joined = arg;
        else joined += '/' + arg;
      }
    }
    return posix.normalize(joined);
  },

  relative(from, to) {
    from = posix.resolve(from);
    to = posix.resolve(to);
    if (from === to) return '';
    const fromParts = from.split('/').filter(Boolean);
    const toParts = to.split('/').filter(Boolean);
    let i = 0;
    while (i < fromParts.length && i < toParts.length && fromParts[i] === toParts[i]) i++;
    const up = fromParts.length - i;
    const rel = [];
    for (let j = 0; j < up; j++) rel.push('..');
    rel.push(...toParts.slice(i));
    return rel.join('/') || '.';
  },

  dirname(path) {
    if (path.length === 0) return '.';
    const hasRoot = path.charCodeAt(0) === CHAR_FORWARD_SLASH;
    let end = -1;
    let matchedSlash = true;
    for (let i = path.length - 1; i >= 1; i--) {
      if (path.charCodeAt(i) === CHAR_FORWARD_SLASH) {
        if (!matchedSlash) { end = i; break; }
      } else {
        matchedSlash = false;
      }
    }
    if (end === -1) return hasRoot ? '/' : '.';
    if (hasRoot && end === 1) return '/';
    return path.slice(0, end);
  },

  basename(path, ext) {
    if (path.length === 0) return path;
    let start = 0;
    let end = -1;
    let matchedSlash = true;
    for (let i = path.length - 1; i >= 0; i--) {
      if (path.charCodeAt(i) === CHAR_FORWARD_SLASH) {
        if (!matchedSlash) { start = i + 1; break; }
      } else if (end === -1) {
        matchedSlash = false;
        end = i + 1;
      }
    }
    if (end === -1) return '';
    const basename = path.slice(start, end);
    if (ext && ext.length > 0 && basename.endsWith(ext)) {
      return basename.slice(0, basename.length - ext.length);
    }
    return basename;
  },

  extname(path) {
    if (path.length === 0) return '';
    let startDot = -1;
    let startPart = 0;
    let end = -1;
    let matchedSlash = true;
    let preDotState = 0;
    for (let i = path.length - 1; i >= 0; i--) {
      const code = path.charCodeAt(i);
      if (code === CHAR_FORWARD_SLASH) {
        if (!matchedSlash) { startPart = i + 1; break; }
        continue;
      }
      if (end === -1) {
        matchedSlash = false;
        end = i + 1;
      }
      if (code === CHAR_DOT) {
        if (startDot === -1) startDot = i;
        else if (preDotState !== 1) preDotState = 1;
      } else if (startDot !== -1) {
        preDotState = -1;
      }
    }
    if (startDot === -1 || end === -1 ||
        preDotState === 0 || (preDotState === 1 && startDot - 1 === startPart && startDot - 1 > 0)) {
      return '';
    }
    return path.slice(startDot, end);
  },

  parse(path) {
    const isAbsolute = path.charCodeAt(0) === CHAR_FORWARD_SLASH;
    const dir = posix.dirname(path);
    const base = posix.basename(path);
    const ext = posix.extname(path);
    const name = base.slice(0, base.length - ext.length);
    return { root: isAbsolute ? '/' : '', dir, base, ext, name };
  },

  format(pathObject) {
    if (pathObject === null || typeof pathObject !== 'object') throw new TypeError('Path object must be an object');
    return _format('/', pathObject);
  },

  isAbsolute(path) {
    return path.length > 0 && path.charCodeAt(0) === CHAR_FORWARD_SLASH;
  },
};

function normalizeString(path, allowAboveRoot) {
  let res = '';
  let lastSegmentLength = 0;
  let lastSlash = -1;
  let dots = 0;
  let code = 0;
  for (let i = 0; i <= path.length; i++) {
    if (i < path.length) code = path.charCodeAt(i);
    else if (isPathSeparator(code)) break;
    else code = CHAR_FORWARD_SLASH;
    if (isPathSeparator(code)) {
      if (lastSlash === i - 1 || dots === 1);
      else if (dots === 2) {
        if (res.length < 2 || lastSegmentLength !== 2 ||
            res.charCodeAt(res.length - 1) !== CHAR_DOT ||
            res.charCodeAt(res.length - 2) !== CHAR_DOT) {
          if (res.length > 2) {
            const lastSlashIdx = res.lastIndexOf('/');
            if (lastSlashIdx === -1) { res = ''; lastSegmentLength = 0; }
            else { res = res.slice(0, lastSlashIdx); lastSegmentLength = res.length - res.lastIndexOf('/') - 1; }
          } else if (res.length > 0) { res = ''; lastSegmentLength = 0; }
        }
      } else {
        if (res.length > 0) res += '/';
        res += path.slice(lastSlash + 1, i);
        lastSegmentLength = i - lastSlash - 1;
      }
      lastSlash = i;
      dots = 0;
    } else if (code === CHAR_DOT && dots !== -1) {
      dots++;
    } else {
      dots = -1;
    }
  }
  return res;
}

const win32 = {
  sep: '\\',
  delimiter: ';',
  resolve(...args) {
    let resolvedPath = '';
    let resolvedAbsolute = false;
    for (let i = args.length - 1; i >= -1 && !resolvedAbsolute; i--) {
      const path = i >= 0 ? args[i] : process.cwd();
      if (typeof path !== 'string') throw new TypeError('Path must be a string');
      if (path.length === 0) continue;
      resolvedPath = path + '\\' + resolvedPath;
      resolvedAbsolute = path.charCodeAt(1) === CHAR_FORWARD_SLASH;
    }
    if (resolvedAbsolute) return resolvedPath.replace(/\//g, '\\');
    return resolvedPath.replace(/\//g, '\\');
  },
  normalize(path) {
    if (path.length === 0) return '.';
    return path.replace(/\//g, '\\').replace(/\\\\+/g, '\\').replace(/\\$/, '');
  },
  join(...args) {
    if (args.length === 0) return '.';
    return args.filter(Boolean).join('\\').replace(/\//g, '\\');
  },
  relative(from, to) {
    from = win32.resolve(from);
    to = win32.resolve(to);
    if (from === to) return '';
    return posix.relative(from.replace(/\\/g, '/'), to.replace(/\\/g, '/')).replace(/\//g, '\\');
  },
  dirname(path) {
    return posix.dirname(path.replace(/\\/g, '/')).replace(/\//g, '\\');
  },
  basename(path, ext) {
    return posix.basename(path.replace(/\\/g, '/'), ext);
  },
  extname(path) {
    return posix.extname(path.replace(/\\/g, '/'));
  },
  parse(path) {
    const p = posix.parse(path.replace(/\\/g, '/'));
    return { ...p, root: p.root ? '\\' : '' };
  },
  format(pathObject) {
    return _format('\\', pathObject);
  },
  isAbsolute(path) {
    return path.length > 0 && (path.charCodeAt(1) === CHAR_FORWARD_SLASH);
  },
};

const path = {
  sep: posix.sep,
  delimiter: posix.delimiter,
  ...posix,
  posix,
  win32,
  _format,
};

if (typeof module !== 'undefined' && module.exports) {
  module.exports = path;
}
