/// <reference path="./klyron.d.ts" />

declare namespace Klyron {
  // ── File System API ──────────────────────────────────────────────

  namespace fs {
    function readFile(path: string | URL): Promise<Uint8Array>;
    function readTextFile(path: string | URL): Promise<string>;
    function writeFile(path: string | URL, data: Uint8Array | string): Promise<void>;
    function writeTextFile(path: string | URL, data: string): Promise<void>;
    function remove(path: string | URL, options?: { recursive?: boolean }): Promise<void>;
    function copyFile(from: string | URL, to: string | URL): Promise<void>;
    function moveFile(from: string | URL, to: string | URL): Promise<void>;
    function exists(path: string | URL): Promise<boolean>;
    function mkdir(path: string | URL, options?: { recursive?: boolean; mode?: number }): Promise<void>;
    function readDir(path: string | URL): AsyncIterable<Klyron.ReadDirEntry>;
    function lstat(path: string | URL): Promise<Klyron.FileInfo>;
    function stat(path: string | URL): Promise<Klyron.FileInfo>;
    function chmod(path: string | URL, mode: number): Promise<void>;
    function chown(path: string | URL, uid: number, gid: number): Promise<void>;
    function link(oldPath: string, newPath: string): Promise<void>;
    function symlink(target: string | URL, path: string | URL): Promise<void>;
    function readLink(path: string | URL): Promise<string>;
    function realPath(path: string | URL): Promise<string>;
    function truncate(path: string | URL, len?: number): Promise<void>;
    function utime(path: string | URL, atime: Date | number, mtime: Date | number): Promise<void>;
    function makeTempFile(options?: { dir?: string; prefix?: string; suffix?: string }): Promise<{ path: string; file: { write(data: Uint8Array): Promise<void>; close(): void } }>;
    function makeTempDir(options?: { dir?: string; prefix?: string; suffix?: string }): Promise<string>;
  }

  // ── HTTP Client API ──────────────────────────────────────────────

  namespace http {
    interface RequestOptions {
      method?: string;
      headers?: Record<string, string>;
      body?: string | Uint8Array;
      signal?: AbortSignal;
      timeout?: number;
      followRedirects?: boolean;
      maxRedirects?: number;
    }

    interface HttpResponse {
      status: number;
      statusText: string;
      headers: Record<string, string>;
      body: Uint8Array;
      json(): Promise<any>;
      text(): Promise<string>;
      arrayBuffer(): Promise<ArrayBuffer>;
      blob(): Promise<Blob>;
    }

    function fetch(url: string | URL, options?: RequestOptions): Promise<HttpResponse>;
    function get(url: string | URL, options?: RequestOptions): Promise<HttpResponse>;
    function post(url: string | URL, body?: string | Uint8Array, options?: RequestOptions): Promise<HttpResponse>;
    function put(url: string | URL, body?: string | Uint8Array, options?: RequestOptions): Promise<HttpResponse>;
    function patch(url: string | URL, body?: string | Uint8Array, options?: RequestOptions): Promise<HttpResponse>;
    function delete(url: string | URL, options?: RequestOptions): Promise<HttpResponse>;
    function head(url: string | URL, options?: RequestOptions): Promise<HttpResponse>;
  }

  // ── HTTP Server API ──────────────────────────────────────────────

  namespace serve {
    interface ServeOptions {
      port: number;
      hostname?: string;
      reusePort?: boolean;
      reuseAddress?: boolean;
      cert?: string;
      key?: string;
    }

    interface ServeHandler {
      (request: Request): Response | Promise<Response>;
    }

    interface HttpServer {
      readonly addr: { hostname: string; port: number; transport: "tcp" };
      close(): Promise<void>;
    }

    function serve(handler: ServeHandler, options?: ServeOptions): Promise<HttpServer>;
    function listenAndServe(addr: string, handler: ServeHandler): Promise<HttpServer>;
  }

  // ── WebSocket API ────────────────────────────────────────────────

  namespace ws {
    interface WebSocketOptions {
      headers?: Record<string, string>;
      protocols?: string[];
      signal?: AbortSignal;
    }

    function connect(url: string | URL, options?: WebSocketOptions): Promise<WebSocket>;
  }

  // ── Process / Subprocess API ─────────────────────────────────────

  namespace process {
    interface RunOptions {
      cmd: string[];
      cwd?: string;
      env?: Record<string, string>;
      stdin?: "inherit" | "piped" | "null";
      stdout?: "inherit" | "piped" | "null";
      stderr?: "inherit" | "piped" | "null";
      signal?: AbortSignal;
      timeout?: number;
    }

    interface CommandResult {
      readonly pid: number;
      readonly status: Promise<{ code: number; signal?: string }>;
      readonly stdin: WritableStream;
      readonly stdout: ReadableStream;
      readonly stderr: ReadableStream;
      kill(signal?: string): void;
      output(): Promise<{ code: number; stdout: Uint8Array; stderr: Uint8Array }>;
    }

    function run(options: RunOptions): CommandResult;
    function exec(cmd: string, args?: string[]): Promise<{ code: number; stdout: string; stderr: string }>;
    function spawn(cmd: string, args?: string[]): { pid: number; kill(signal?: string): void };
  }

  // ── Crypto API ───────────────────────────────────────────────────

  namespace crypto {
    function randomBytes(size: number): Uint8Array;
    function randomUUID(): string;
    function md5(data: string | Uint8Array): string;
    function sha1(data: string | Uint8Array): string;
    function sha256(data: string | Uint8Array): string;
    function sha512(data: string | Uint8Array): string;

    interface KeyPair {
      publicKey: CryptoKey;
      privateKey: CryptoKey;
    }

    function generateKeyPair(algorithm: "rsa" | "ec" | "ed25519" | "x25519"): Promise<KeyPair>;
  }

  // ── Compression API ──────────────────────────────────────────────

  namespace compress {
    function gzip(data: Uint8Array, level?: number): Promise<Uint8Array>;
    function gunzip(data: Uint8Array): Promise<Uint8Array>;
    function deflate(data: Uint8Array, level?: number): Promise<Uint8Array>;
    function inflate(data: Uint8Array): Promise<Uint8Array>;
    function brotliCompress(data: Uint8Array, quality?: number): Promise<Uint8Array>;
    function brotliDecompress(data: Uint8Array): Promise<Uint8Array>;
  }

  // ── Path API ─────────────────────────────────────────────────────

  namespace path {
    function join(...paths: string[]): string;
    function resolve(...paths: string[]): string;
    function normalize(path: string): string;
    function relative(from: string, to: string): string;
    function dirname(path: string): string;
    function basename(path: string, suffix?: string): string;
    function extname(path: string): string;
    function parse(path: string): { root: string; dir: string; base: string; ext: string; name: string };
    function format(pathObject: { root?: string; dir?: string; base?: string; ext?: string; name?: string }): string;
    function isAbsolute(path: string): boolean;
    function sep(): string;
    function delimiter(): string;
    function fromFileUrl(url: string | URL): string;
    function toFileUrl(path: string): URL;
  }

  // ── OS / System API ──────────────────────────────────────────────

  namespace os {
    function hostname(): string;
    function platform(): string;
    function arch(): string;
    function release(): string;
    function type(): string;
    function uptime(): number;
    function loadavg(): number[];
    function cpus(): { model: string; speed: number; times: { user: number; nice: number; sys: number; idle: number; irq: number } }[];
    function totalmem(): number;
    function freemem(): number;
    function networkInterfaces(): Record<string, { address: string; netmask: string; family: string; mac: string; internal: boolean; cidr: string }[]>;
    function homedir(): string;
    function tmpdir(): string;
    function userInfo(): { username: string; uid: number; gid: number; shell: string | null; homedir: string };
    function endianness(): "LE" | "BE";
    function availableParallelism(): number;
  }

  // ── Runtime API ──────────────────────────────────────────────────

  namespace runtime {
    interface MemoryUsage {
      rss: number;
      heapTotal: number;
      heapUsed: number;
      external: number;
    }

    interface CpuUsage {
      user: number;
      system: number;
    }

    function memoryUsage(): MemoryUsage;
    function cpuUsage(): CpuUsage;
    function resourceUsage(): Klyron.ResourceUsage;
    function metrics(): Klyron.Metrics;
    function gc(): void;
    function version(): string;
    function buildInfo(): Klyron.BuildInfo;
    function exit(code?: number): never;
    function setExitHandler(handler: (code: number) => void | Promise<void>): void;
  }

  // ── DNS API ──────────────────────────────────────────────────────

  namespace dns {
    interface DnsRecord {
      address: string;
      family: number;
      ttl: number;
    }

    function resolve(hostname: string, recordType?: "A" | "AAAA" | "CNAME" | "MX" | "TXT" | "SRV" | "NS" | "SOA"): Promise<DnsRecord[]>;
    function resolve4(hostname: string, options?: { signal?: AbortSignal }): Promise<string[]>;
    function resolve6(hostname: string, options?: { signal?: AbortSignal }): Promise<string[]>;
    function resolveCname(hostname: string): Promise<string[]>;
    function resolveMx(hostname: string): Promise<{ exchange: string; priority: number }[]>;
    function resolveTxt(hostname: string): Promise<string[][]>;
    function resolveSrv(hostname: string): Promise<{ name: string; port: number; priority: number; weight: number }[]>;
    function resolveNs(hostname: string): Promise<string[]>;
    function reverse(ip: string): Promise<string[]>;
    function lookup(hostname: string, options?: { family?: number; hints?: number; all?: boolean }): Promise<{ address: string; family: number } | { address: string; family: number }[]>;
  }

  // ── Console Extensions ───────────────────────────────────────────

  namespace console {
    function time(label?: string): void;
    function timeEnd(label?: string): number;
    function timeLog(label?: string, ...data: any[]): void;
    function table(tabularData?: any, properties?: string[]): void;
    function group(...data: any[]): void;
    function groupCollapsed(...data: any[]): void;
    function groupEnd(): void;
  }

  // ── Assert API ───────────────────────────────────────────────────

  namespace assert {
    function ok(value: unknown, message?: string): asserts value;
    function equal<T>(actual: T, expected: T, message?: string): void;
    function notEqual<T>(actual: T, expected: T, message?: string): void;
    function strictEqual<T>(actual: T, expected: T, message?: string): asserts actual is T;
    function notStrictEqual<T>(actual: T, expected: T, message?: string): void;
    function deepEqual<T>(actual: T, expected: T, message?: string): void;
    function notDeepEqual<T>(actual: T, expected: T, message?: string): void;
    function throws(fn: () => void, error?: RegExp | Function | object, message?: string): void;
    function doesNotThrow(fn: () => void, message?: string): void;
    function rejects(fn: () => Promise<any>, error?: RegExp | Function | object, message?: string): Promise<void>;
    function doesNotReject(fn: () => Promise<any>, message?: string): Promise<void>;
    function fail(message?: string): never;
    function ifError(value: unknown): asserts value is null | undefined;
    function match(str: string, regexp: RegExp, message?: string): void;
    function doesNotMatch(str: string, regexp: RegExp, message?: string): void;
    function snapshot<T>(value: T, name?: string): void;
  }

  // ── Testing API ──────────────────────────────────────────────────

  namespace test {
    interface TestOptions {
      name: string;
      fn: () => void | Promise<void>;
      ignore?: boolean;
      only?: boolean;
      timeout?: number;
      sanitizeOps?: boolean;
      sanitizeResources?: boolean;
      sanitizeExit?: boolean;
    }

    interface TestRunner {
      run(): Promise<{ passed: number; failed: number; ignored: number; total: number; duration: number }>;
      start(): void;
      end(): Promise<{ passed: number; failed: number; ignored: number; total: number; duration: number }>;
    }

    function test(options: TestOptions): void;
    function test(name: string, fn: () => void | Promise<void>): void;
    function describe(name: string, fn: () => void): void;
    function it(name: string, fn: () => void | Promise<void>): void;
    function before(fn: () => void | Promise<void>): void;
    function after(fn: () => void | Promise<void>): void;
    function beforeEach(fn: () => void | Promise<void>): void;
    function afterEach(fn: () => void | Promise<void>): void;
    function mock<T extends Function>(fn?: T): T;
    function spy<T extends Function>(obj: any, method: string): { calls: { args: any[]; result: any }[]; restore(): void };
  }

  // ── Encoding API ─────────────────────────────────────────────────

  namespace encoding {
    function atob(data: string): string;
    function btoa(data: string): string;
    namespace hex {
      function encode(data: Uint8Array): string;
      function decode(data: string): Uint8Array;
    }
    namespace base64 {
      function encode(data: Uint8Array): string;
      function decode(data: string): Uint8Array;
      function encodeUrl(data: Uint8Array): string;
      function decodeUrl(data: string): Uint8Array;
    }
    namespace base32 {
      function encode(data: Uint8Array): string;
      function decode(data: string): Uint8Array;
    }
  }

  // ── UUID API ─────────────────────────────────────────────────────

  namespace uuid {
    function v4(): string;
    function v7(): string;
    function validate(uuid: string): boolean;
    function version(uuid: string): number;
    function nil(): string;
  }

  // ── Semver API ───────────────────────────────────────────────────

  namespace semver {
    interface SemVer {
      major: number;
      minor: number;
      patch: number;
      prerelease: string[];
      build: string[];
      version: string;
    }

    function parse(version: string): SemVer | null;
    function valid(version: string): string | null;
    function clean(version: string): string | null;
    function satisfies(version: string, range: string): boolean;
    function maxSatisfying(versions: string[], range: string): string | null;
    function minSatisfying(versions: string[], range: string): string | null;
    function sort(versions: string[]): string[];
    function rsort(versions: string[]): string[];
    function compare(v1: string, v2: string): number;
    function rcompare(v1: string, v2: string): number;
    function diff(v1: string, v2: string): string | null;
    function inc(version: string, release: "major" | "minor" | "patch" | "premajor" | "preminor" | "prepatch" | "prerelease"): string;
    function coerce(version: string): SemVer | null;
    function minVersion(range: string): string | null;
    function intersects(range1: string, range2: string): boolean;
    function gtr(version: string, range: string): boolean;
    function ltr(version: string, range: string): boolean;
    function outside(version: string, range: string, hilo: ">" | "<"): boolean;
    function prerelease(version: string): string[] | null;
    function major(version: string): number;
    function minor(version: string): number;
    function patch(version: string): number;
  }

  // ── DateTime API ─────────────────────────────────────────────────

  namespace datetime {
    interface DateTime {
      year: number;
      month: number;
      day: number;
      hour: number;
      minute: number;
      second: number;
      millisecond: number;
      microsecond: number;
      nanosecond: number;
    }

    function now(): DateTime;
    function utcNow(): DateTime;
    function parse(dateString: string, format?: string): DateTime;
    function format(date: DateTime, format: string): string;
    function fromMillis(ms: number): DateTime;
    function toMillis(date: DateTime): number;
    function fromISO(iso: string): DateTime;
    function toISO(date: DateTime): string;
    function add(date: DateTime, duration: Duration): DateTime;
    function subtract(date: DateTime, duration: Duration): DateTime;
    function diff(d1: DateTime, d2: DateTime): Duration;

    interface Duration {
      years?: number;
      months?: number;
      weeks?: number;
      days?: number;
      hours?: number;
      minutes?: number;
      seconds?: number;
      milliseconds?: number;
      microseconds?: number;
      nanoseconds?: number;
    }

    function duration(d: Duration): Duration;
  }

  // ── Logger API ───────────────────────────────────────────────────

  namespace log {
    enum LogLevel {
      NOTSET = 0,
      DEBUG = 10,
      INFO = 20,
      WARN = 30,
      ERROR = 40,
      CRITICAL = 50,
    }

    interface LogConfig {
      level?: LogLevel;
      format?: string;
      handlers?: LogHandler[];
    }

    interface LogHandler {
      log(record: LogRecord): void;
    }

    interface LogRecord {
      msg: string;
      level: LogLevel;
      logger: string;
      timestamp: Date;
      args: any[];
    }

    function debug(msg: string, ...args: any[]): void;
    function info(msg: string, ...args: any[]): void;
    function warn(msg: string, ...args: any[]): void;
    function error(msg: string, ...args: any[]): void;
    function critical(msg: string, ...args: any[]): void;
    function getLogger(name?: string): Logger;
    function setLevel(level: LogLevel): void;
    function setConfig(config: LogConfig): void;
  }

  interface Logger {
    debug(msg: string, ...args: any[]): void;
    info(msg: string, ...args: any[]): void;
    warn(msg: string, ...args: any[]): void;
    error(msg: string, ...args: any[]): void;
    critical(msg: string, ...args: any[]): void;
  }

  // ── Stream API ───────────────────────────────────────────────────

  namespace stream {
    function readableFrom<T>(iterable: Iterable<T> | AsyncIterable<T>): ReadableStream<T>;
    function writableTo<T>(fn: (chunk: T) => void | Promise<void>): WritableStream<T>;
    function transform<T, R>(transformer: (chunk: T) => R | Promise<R>): TransformStream<T, R>;
    function concat(...streams: ReadableStream[]): ReadableStream;
    function merge(...streams: ReadableStream[]): ReadableStream;
    function pipeline<T>(readable: ReadableStream<T>, ...transforms: TransformStream[]): ReadableStream;
    function readAll<T>(stream: ReadableStream<T>): Promise<T[]>;
    function writeAll<T>(stream: WritableStream<T>, data: T[]): Promise<void>;
    function intoAsyncIterable<T>(stream: ReadableStream<T>): AsyncIterable<T>;
  }

  // ── Glob API ─────────────────────────────────────────────────────

  namespace glob {
    interface GlobOptions {
      root?: string;
      exclude?: string[];
      caseInsensitive?: boolean;
      followSymlinks?: boolean;
      maxDepth?: number;
    }

    function glob(pattern: string, options?: GlobOptions): AsyncIterable<string>;
    function globSync(pattern: string, options?: GlobOptions): string[];
    function expandGlob(pattern: string, options?: GlobOptions): AsyncIterable<string>;
    function expandGlobSync(pattern: string, options?: GlobOptions): string[];
    function matchGlob(path: string, pattern: string): boolean;
  }

  // ── Cache API ────────────────────────────────────────────────────

  namespace cache {
    interface CacheEntry<T> {
      value: T;
      expiresAt: number;
    }

    interface CacheOptions {
      ttl?: number;
      namespace?: string;
    }

    function get<T>(key: string): Promise<T | undefined>;
    function set<T>(key: string, value: T, options?: CacheOptions): Promise<void>;
    function delete(key: string): Promise<boolean>;
    function clear(): Promise<void>;
    function has(key: string): Promise<boolean>;
    function keys(): Promise<string[]>;
    function size(): Promise<number>;
    function memo<T>(key: string, fn: () => Promise<T>, options?: CacheOptions): Promise<T>;
  }

  // ── Plugin API ───────────────────────────────────────────────────

  namespace plugin {
    interface PluginManifest {
      name: string;
      version: string;
      description?: string;
      entry: string;
      permissions?: string[];
      dependencies?: Record<string, string>;
    }

    function load(path: string | URL): Promise<PluginManifest>;
    function unload(name: string): Promise<void>;
    function list(): PluginManifest[];
    function call(name: string, method: string, ...args: any[]): Promise<any>;
  }
}

// Top-level Klyron APIs
declare var Klyron: {
  fs: typeof Klyron.fs;
  http: typeof Klyron.http;
  serve: typeof Klyron.serve;
  ws: typeof Klyron.ws;
  process: typeof Klyron.process;
  crypto: typeof Klyron.crypto;
  compress: typeof Klyron.compress;
  path: typeof Klyron.path;
  os: typeof Klyron.os;
  runtime: typeof Klyron.runtime;
  dns: typeof Klyron.dns;
  assert: typeof Klyron.assert;
  test: typeof Klyron.test;
  encoding: typeof Klyron.encoding;
  uuid: typeof Klyron.uuid;
  semver: typeof Klyron.semver;
  datetime: typeof Klyron.datetime;
  log: typeof Klyron.log;
  stream: typeof Klyron.stream;
  glob: typeof Klyron.glob;
  cache: typeof Klyron.cache;
  plugin: typeof Klyron.plugin;
};

// fetch is globally available
declare function fetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response>;

// setTimeout/setInterval globals
declare function setTimeout(handler: TimerHandler, timeout?: number, ...arguments: any[]): number;
declare function clearTimeout(timeoutId: number | undefined): void;
declare function setInterval(handler: TimerHandler, timeout?: number, ...arguments: any[]): number;
declare function clearInterval(intervalId: number | undefined): void;
declare function setImmediate(handler: TimerHandler, ...arguments: any[]): number;
declare function clearImmediate(immediateId: number | undefined): void;

type TimerHandler = string | Function;

declare function queueMicrotask(callback: () => void): void;
declare function structuredClone<T>(value: T, options?: StructuredSerializeOptions): T;

interface QueueMicrotask {
  (callback: () => void): void;
}

interface StructuredSerializeOptions {
  transfer?: Transferable[];
}

// Atomics for SharedArrayBuffer
interface Atomics {
  add(typedArray: Int8Array | Uint8Array | Int16Array | Uint16Array | Int32Array | Uint32Array, index: number, value: number): number;
  and(typedArray: Int8Array | Uint8Array | Int16Array | Uint16Array | Int32Array | Uint32Array, index: number, value: number): number;
  compareExchange(typedArray: Int8Array | Uint8Array | Int16Array | Uint16Array | Int32Array | Uint32Array, index: number, expectedValue: number, replacementValue: number): number;
  exchange(typedArray: Int8Array | Uint8Array | Int16Array | Uint16Array | Int32Array | Uint32Array, index: number, value: number): number;
  isLockFree(size: number): boolean;
  load(typedArray: Int8Array | Uint8Array | Int16Array | Uint16Array | Int32Array | Uint32Array, index: number): number;
  notify(typedArray: Int32Array | BigInt64Array, index: number, count?: number): number;
  or(typedArray: Int8Array | Uint8Array | Int16Array | Uint16Array | Int32Array | Uint32Array, index: number, value: number): number;
  store(typedArray: Int8Array | Uint8Array | Int16Array | Uint16Array | Int32Array | Uint32Array, index: number, value: number): number;
  sub(typedArray: Int8Array | Uint8Array | Int16Array | Uint16Array | Int32Array | Uint32Array, index: number, value: number): number;
  wait(typedArray: Int32Array | BigInt64Array, index: number, value: number, timeout?: number): "ok" | "not-equal" | "timed-out";
  waitAsync(typedArray: Int32Array | BigInt64Array, index: number, value: number, timeout?: number): { async: false; value: string } | { async: true; value: Promise<string> };
  xor(typedArray: Int8Array | Uint8Array | Int16Array | Uint16Array | Int32Array | Uint32Array, index: number, value: number): number;
}

interface SharedArrayBuffer {
  readonly byteLength: number;
  slice(begin: number, end?: number): SharedArrayBuffer;
}

// WebAssembly types
declare namespace WebAssembly {
  interface Module {}
  interface Instance {
    readonly exports: Record<string, any>;
  }
  interface Memory {
    readonly buffer: ArrayBuffer;
    grow(delta: number): number;
  }
  interface Table {
    readonly length: number;
    get(index: number): any;
    set(index: number, value: any): void;
    grow(delta: number, initValue?: any): number;
  }
  interface CompileError extends Error {}
  interface LinkError extends Error {}
  interface RuntimeError extends Error {}

  type Imports = Record<string, Record<string, Function>>;
  type ExportValue = Function | Global | Memory | Table;

  interface Global {
    value: any;
    valueOf(): any;
  }

  function compile(bytes: BufferSource): Promise<Module>;
  function compileStreaming(source: Response | Promise<Response>): Promise<Module>;
  function instantiate(bytes: BufferSource, importObject?: Imports): Promise<{ module: Module; instance: Instance }>;
  function instantiate(module: Module, importObject?: Imports): Promise<Instance>;
  function instantiateStreaming(source: Response | Promise<Response>, importObject?: Imports): Promise<{ module: Module; instance: Instance }>;
  function validate(bytes: BufferSource): boolean;
  function decode(bytes: BufferSource): Module;

  namespace instantiateStreaming {
    // overloads
  }
}

declare var WebAssembly: typeof WebAssembly;
