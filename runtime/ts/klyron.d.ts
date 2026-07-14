/// <reference path="./globals.d.ts" />
/// <reference path="./runtime.d.ts" />

declare namespace Klyron {
  interface BuildInfo {
    version: string;
    buildTime: string;
    commit: string;
    rustc: string;
    target: string;
  }

  interface ResourceUsage {
    cpuTime: number;
    memoryUsage: number;
    peakMemory: number;
    openFiles: number;
    openConnections: number;
  }

  interface Metrics {
    ops: number;
    bytesSent: number;
    bytesReceived: number;
  }
}

declare var klyronBuildInfo: Klyron.BuildInfo;

interface ImportMeta {
  url: string;
  main: boolean;
  resolve(specifier: string): string;
  dirname: string;
  filename: string;
  env: Record<string, string | undefined>;
}

declare namespace NodeJS {
  interface ProcessVersions {
    klyron: string;
    node?: string;
    v8?: string;
    uv?: string;
    zlib?: string;
    brotli?: string;
    ares?: string;
    modules?: string;
    nghttp2?: string;
    napi?: string;
    openssl?: string;
    http_parser?: string;
  }

  interface ProcessEnv {
    [key: string]: string | undefined;
  }

  interface Process {
    env: ProcessEnv;
    argv: string[];
    pid: number;
    ppid: number;
    cwd(): string;
    chdir(directory: string): void;
    exit(code?: number): never;
    versions: ProcessVersions;
    platform: string;
    arch: string;
    uptime(): number;
    hrtime(): [number, number];
    memoryUsage(): { rss: number; heapTotal: number; heapUsed: number; external: number };
    nextTick(callback: Function, ...args: any[]): void;
    on(event: string, listener: (...args: any[]) => void): this;
    once(event: string, listener: (...args: any[]) => void): this;
    emit(event: string, ...args: any[]): boolean;
    stderr: { write(data: string): boolean };
    stdout: { write(data: string): boolean };
    stdin: { read(): Promise<string | null> };
  }

  interface ProcessModule {
    stdout: { write(data: string): boolean };
    stderr: { write(data: string): boolean };
    stdin: { read(): Promise<string | null> };
  }
}

declare var process: NodeJS.Process;

declare var Buffer: typeof import("buffer").Buffer;

declare namespace Klyron {
  interface ReadDirEntry {
    name: string;
    isFile: boolean;
    isDirectory: boolean;
    isSymlink: boolean;
  }

  interface FileInfo {
    size: number;
    modified: Date;
    accessed: Date;
    created: Date;
    isFile: boolean;
    isDirectory: boolean;
    isSymlink: boolean;
    mode: number;
  }
}

interface Console {
  assert(condition?: boolean, ...data: any[]): void;
  clear(): void;
  count(label?: string): void;
  countReset(label?: string): void;
  debug(...data: any[]): void;
  dir(item?: any, options?: any): void;
  dirxml(...data: any[]): void;
  error(...data: any[]): void;
  group(...data: any[]): void;
  groupCollapsed(...data: any[]): void;
  groupEnd(): void;
  info(...data: any[]): void;
  log(...data: any[]): void;
  table(tabularData?: any, properties?: string[]): void;
  time(label?: string): void;
  timeEnd(label?: string): void;
  timeLog(label?: string, ...data: any[]): void;
  timeStamp(label?: string): void;
  trace(...data: any[]): void;
  warn(...data: any[]): void;
}

declare var console: Console;

interface ErrorConstructor {
  captureStackTrace(targetObject: object, constructorOpt?: Function): void;
  prepareStackTrace?: (err: Error, stackTraces: NodeJS.CallSite[]) => any;
  stackTraceLimit: number;
}

declare namespace NodeJS {
  interface CallSite {
    getThis(): unknown;
    getTypeName(): string | null;
    getFunction(): Function | undefined;
    getFunctionName(): string | null;
    getMethodName(): string | null;
    getFileName(): string | null;
    getLineNumber(): number | null;
    getColumnNumber(): number | null;
    getEvalOrigin(): string | undefined;
    isToplevel(): boolean;
    isEval(): boolean;
    isNative(): boolean;
    isConstructor(): boolean;
    isAsync(): boolean;
    isPromiseAll(): boolean;
    getPromiseIndex(): number | null;
  }
}

declare var __filename: string;
declare var __dirname: string;
