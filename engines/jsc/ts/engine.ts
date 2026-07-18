import type {
  JSCValueType,
  JSCHeapStats,
  JSCResult,
  JSCStringResult,
} from "./types.ts";

export interface NativeJSCBindings {
  jscInit(): number;
  jscShutdown(handle: number): void;

  jscEval(handle: number, code: string): JSCStringResult;
  jscExecuteScript(handle: number, filename: string, source: string): JSCStringResult;
  jscExecuteModule(handle: number, filename: string, source: string): JSCStringResult;

  jscJsonStringify(handle: number, valueHandle: number): JSCStringResult;
  jscJsonParse(handle: number, json: string): number;

  jscSetGlobal(handle: number, name: string, valueHandle: number): JSCResult;
  jscGetGlobal(handle: number, name: string): number;

  jscValueNewString(handle: number, str: string): number;
  jscValueNewNumber(handle: number, num: number): number;
  jscValueNewBool(handle: number, val: boolean): number;
  jscValueNewNull(handle: number): number;
  jscValueNewUndefined(handle: number): number;
  jscValueNewObject(handle: number): number;
  jscValueNewArray(handle: number): number;

  jscValueTypeof(handle: number, valueHandle: number): { type: JSCValueType; success: boolean };
  jscValueToString(handle: number, valueHandle: number): JSCStringResult;
  jscValueToNumber(handle: number, valueHandle: number): number;
  jscValueToBool(handle: number, valueHandle: number): boolean;
  jscValueIsArray(handle: number, valueHandle: number): boolean;
  jscValueDispose(valueHandle: number): void;

  jscPromiseNew(handle: number): number;
  jscPromiseResolve(handle: number, promiseHandle: number, valueHandle: number): JSCResult;
  jscPromiseReject(handle: number, promiseHandle: number, reason: string): JSCResult;

  jscModuleCompile(handle: number, source: string, origin: string): number;
  jscModuleInstantiate(handle: number, moduleHandle: number): JSCResult;
  jscModuleEvaluate(handle: number, moduleHandle: number): JSCStringResult;
  jscModuleDispose(moduleHandle: number): void;

  jscGetHeapStats(handle: number): JSCHeapStats;
  jscRequestGC(handle: number): void;
  jscLowMemoryNotification(handle: number): void;

  jscMicrotasksPerformCheck(handle: number): void;

  jscGetExceptionMessage(handle: number): string;
  jscGetStackTrace(handle: number): JSCStringResult;

  jscVersion(): string;
  jscFreeString(s: string): void;
  jscFreeBuffer(buf: Uint8Array): void;
  jscIsolateEnter(handle: number): void;
  jscIsolateExit(handle: number): void;
  jscContextEnter(handle: number): void;
  jscContextExit(handle: number): void;

  jscBufferAlloc(handle: number, size: number): number;
  jscBufferFromString(handle: number, str: string): number;
  jscBufferConcat(handle: number, bufs: number[]): number;
  jscBufferToString(handle: number, buf: number, encoding: string): JSCStringResult;
  jscBufferSlice(handle: number, buf: number, start: number, end: number): number;

  jscConsoleLog(handle: number, args: number[]): void;
  jscConsoleWarn(handle: number, args: number[]): void;
  jscConsoleError(handle: number, args: number[]): void;
  jscConsoleInfo(handle: number, args: number[]): void;
  jscConsoleDebug(handle: number, args: number[]): void;
  jscConsoleTable(handle: number, data: number): void;
  jscConsoleAssert(handle: number, condition: number, args: number[]): void;
  jscConsoleCount(handle: number, label: string | null): void;
  jscConsoleTime(handle: number, label: string | null): void;
  jscConsoleTimeEnd(handle: number, label: string | null): void;
  jscConsoleTrace(handle: number): void;
  jscConsoleGroup(handle: number, label: string | null): void;
  jscConsoleGroupEnd(handle: number): void;

  jscRandomBytes(handle: number, size: number): number;
  jscRandomFill(handle: number, buf: number, offset: number, size: number): number;
  jscRandomUUID(handle: number): string;

  jscEncodeText(handle: number, input: string): JSCStringResult;
  jscEncodeInto(handle: number, input: string, dst: number, offset: number): number;
  jscDecodeText(handle: number, buf: number, encoding: string): string;
  jscBase64Encode(handle: number, input: string): JSCStringResult;
  jscBase64Decode(handle: number, input: string): number;
  jscHexEncode(handle: number, input: string): JSCStringResult;
  jscHexDecode(handle: number, input: string): number;

  jscSetTimeout(handle: number, callback: number, ms: number): number;
  jscSetInterval(handle: number, callback: number, ms: number): number;
  jscSetImmediate(handle: number, callback: number): number;
  jscClearTimeout(handle: number, id: number): void;
  jscClearInterval(handle: number, id: number): void;
  jscClearImmediate(handle: number, id: number): void;

  jscURLParse(handle: number, url: string): number;
  jscURLResolve(handle: number, base: string, relative: string): JSCStringResult;
  jscURLFormat(handle: number, urlObj: number): string;
  jscURLDomainToASCII(handle: number, domain: string): JSCStringResult;

  jscFSReadFile(handle: number, path: string): number;
  jscFSWriteFile(handle: number, path: string, data: number): JSCResult;
  jscFSStat(handle: number, path: string): any;
  jscFSLstat(handle: number, path: string): any;
  jscFSMkdir(handle: number, path: string, mode: number): JSCResult;
  jscFSMkdirp(handle: number, path: string, mode: number): JSCResult;
  jscFSReaddir(handle: number, path: string): string[];
  jscFSUnlink(handle: number, path: string): JSCResult;
  jscFSRmdir(handle: number, path: string): JSCResult;
  jscFSRename(handle: number, oldPath: string, newPath: string): JSCResult;
  jscFSChmod(handle: number, path: string, mode: number): JSCResult;
  jscFSRealpath(handle: number, path: string): string;
  jscFSExists(handle: number, path: string): boolean;

  jscProcessPid(handle: number): number;
  jscProcessPpid(handle: number): number;
  jscProcessCwdStr(handle: number): JSCStringResult;
  jscProcessExecPath(handle: number): JSCStringResult;
  jscProcessPlatform(handle: number): string;
  jscProcessArch(handle: number): string;
  jscProcessTitle(handle: number): JSCStringResult;
  jscProcessArgv(handle: number): string[];
  jscProcessEnvAll(handle: number): Record<string, string>;
  jscProcessExit(handle: number, code: number): void;
  jscProcessMemoryUsage(handle: number): any;
  jscProcessUptime(handle: number): number;

  jscStreamReadableNew(handle: number, options: number): number;
  jscStreamWritableNew(handle: number, options: number): number;
  jscStreamTransformNew(handle: number, options: number): number;
  jscStreamPush(handle: number, stream: number, chunk: number): JSCResult;
  jscStreamRead(handle: number, stream: number, size: number): number;
  jscStreamEnd(handle: number, stream: number): JSCResult;
  jscStreamDestroy(handle: number, stream: number): JSCResult;
  jscStreamPipe(handle: number, readable: number, writable: number): number;

  jscPathBasename(handle: number, path: string): JSCStringResult;
  jscPathDirname(handle: number, path: string): JSCStringResult;
  jscPathExtname(handle: number, path: string): JSCStringResult;
  jscPathJoin(handle: number, parts: string[]): JSCStringResult;
  jscPathResolve(handle: number, parts: string[]): JSCStringResult;
  jscPathNormalize(handle: number, path: string): JSCStringResult;
  jscPathRelative(handle: number, from: string, to: string): JSCStringResult;
  jscPathIsAbsolute(handle: number, path: string): boolean;

  jscOSHostname(handle: number): string;
  jscOSPlatform(handle: number): string;
  jscOSArch(handle: number): string;
  jscOSRelease(handle: number): string;
  jscOSType(handle: number): string;
  jscOSUptime(handle: number): number;
  jscOSTotalMemory(handle: number): number;
  jscOSFreeMemory(handle: number): number;
  jscOSCpus(handle: number): number;
  jscOSLoadAvg(handle: number): number[];
  jscOSHomedir(handle: number): string;
  jscOSTmpdir(handle: number): string;
  jscOSUserInfo(handle: number): { uid: number; gid: number; username: string; homedir: string; shell: string };
  jscOSInfo(handle: number): any;
}

export class JSCEngine {
  private handle: number;
  private bindings: NativeJSCBindings;

  constructor(bindings: NativeJSCBindings) {
    this.bindings = bindings;
    this.handle = bindings.jscInit();
    if (!this.handle) {
      throw new Error("JSC engine init failed");
    }
  }

  dispose(): void {
    if (this.handle) {
      this.bindings.jscShutdown(this.handle);
      this.handle = 0;
    }
  }

  get isolate(): JSCIsolateModule {
    return new JSCIsolateModule(this.handle, this.bindings);
  }

  get context(): JSCContextModule {
    return new JSCContextModule(this.handle, this.bindings);
  }

  get script(): JSCScriptModule {
    return new JSCScriptModule(this.handle, this.bindings);
  }

  get json(): JSCJsonModule {
    return new JSCJsonModule(this.handle, this.bindings);
  }

  get global(): JSCGlobalModule {
    return new JSCGlobalModule(this.handle, this.bindings);
  }

  get value(): JSCValueModule {
    return new JSCValueModule(this.handle, this.bindings);
  }

  get promise(): JSCPromiseModule {
    return new JSCPromiseModule(this.handle, this.bindings);
  }

  get module(): JSCModuleModule {
    return new JSCModuleModule(this.handle, this.bindings);
  }

  get heap(): JSCHeapModule {
    return new JSCHeapModule(this.handle, this.bindings);
  }

  get error(): JSCErrorModule {
    return new JSCErrorModule(this.handle, this.bindings);
  }

  get microtask(): JSCMicrotaskModule {
    return new JSCMicrotaskModule(this.handle, this.bindings);
  }

  get utils(): JSCUtilsModule {
    return new JSCUtilsModule(this.bindings);
  }

  get inspector(): JSCInspectorModule {
    return new JSCInspectorModule(this.handle, this.bindings);
  }

  get wasm(): JSCWasmModule {
    return new JSCWasmModule(this.handle, this.bindings);
  }

  get buffer(): JSCBufferModule {
    return new JSCBufferModule(this.handle, this.bindings);
  }

  get console(): JSCConsoleModule {
    return new JSCConsoleModule(this.handle, this.bindings);
  }

  get crypto(): JSCCryptoModule {
    return new JSCCryptoModule(this.handle, this.bindings);
  }

  get encoding(): JSCEncodingModule {
    return new JSCEncodingModule(this.handle, this.bindings);
  }

  get timers(): JSCTimersModule {
    return new JSCTimersModule(this.handle, this.bindings);
  }

  get url(): JSCURLModule {
    return new JSCURLModule(this.handle, this.bindings);
  }

  get fs(): JSCFSModule {
    return new JSCFSModule(this.handle, this.bindings);
  }

  get process(): JSCProcessModule {
    return new JSCProcessModule(this.handle, this.bindings);
  }

  get stream(): JSCStreamModule {
    return new JSCStreamModule(this.handle, this.bindings);
  }

  get path(): JSCPathModule {
    return new JSCPathModule(this.handle, this.bindings);
  }

  get os(): JSCOperatingSystemModule {
    return new JSCOperatingSystemModule(this.handle, this.bindings);
  }
}

class JSCBufferModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  alloc(size: number): number { return this.bindings.jscBufferAlloc(this.handle, size); }
  fromString(str: string): number { return this.bindings.jscBufferFromString(this.handle, str); }
  concat(bufs: number[]): number { return this.bindings.jscBufferConcat(this.handle, bufs); }
  toString(buf: number, encoding?: string): string {
    const r = this.bindings.jscBufferToString(this.handle, buf, encoding ?? "utf8");
    if (!r.success) throw new Error("bufferToString failed");
    return r.data ?? "";
  }
  slice(buf: number, start: number, end: number): number { return this.bindings.jscBufferSlice(this.handle, buf, start, end); }
}

class JSCConsoleModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  log(...args: number[]): void { this.bindings.jscConsoleLog(this.handle, args); }
  warn(...args: number[]): void { this.bindings.jscConsoleWarn(this.handle, args); }
  error(...args: number[]): void { this.bindings.jscConsoleError(this.handle, args); }
  info(...args: number[]): void { this.bindings.jscConsoleInfo(this.handle, args); }
  debug(...args: number[]): void { this.bindings.jscConsoleDebug(this.handle, args); }
  table(data: number): void { this.bindings.jscConsoleTable(this.handle, data); }
  assert(condition: number, ...args: number[]): void { this.bindings.jscConsoleAssert(this.handle, condition, args); }
  count(label?: string): void { this.bindings.jscConsoleCount(this.handle, label ?? null); }
  time(label?: string): void { this.bindings.jscConsoleTime(this.handle, label ?? null); }
  timeEnd(label?: string): void { this.bindings.jscConsoleTimeEnd(this.handle, label ?? null); }
  trace(): void { this.bindings.jscConsoleTrace(this.handle); }
  group(label?: string): void { this.bindings.jscConsoleGroup(this.handle, label ?? null); }
  groupEnd(): void { this.bindings.jscConsoleGroupEnd(this.handle); }
}

class JSCCryptoModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  randomBytes(size: number): number { return this.bindings.jscRandomBytes(this.handle, size); }
  randomFill(buf: number, offset?: number, size?: number): number { return this.bindings.jscRandomFill(this.handle, buf, offset ?? 0, size ?? 0); }
  randomUUID(): string { return this.bindings.jscRandomUUID(this.handle); }
}

class JSCEncodingModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  encodeText(input: string): string { const r = this.bindings.jscEncodeText(this.handle, input); return r.data ?? ""; }
  encodeInto(input: string, dst: number, offset: number): number { return this.bindings.jscEncodeInto(this.handle, input, dst, offset); }
  decodeText(buf: number, encoding?: string): string { return this.bindings.jscDecodeText(this.handle, buf, encoding ?? "utf8"); }
  base64Encode(input: string): string { const r = this.bindings.jscBase64Encode(this.handle, input); return r.data ?? ""; }
  base64Decode(input: string): number { return this.bindings.jscBase64Decode(this.handle, input); }
  hexEncode(input: string): string { const r = this.bindings.jscHexEncode(this.handle, input); return r.data ?? ""; }
  hexDecode(input: string): number { return this.bindings.jscHexDecode(this.handle, input); }
}

class JSCTimersModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  setTimeout(cb: number, ms: number): number { return this.bindings.jscSetTimeout(this.handle, cb, ms); }
  setInterval(cb: number, ms: number): number { return this.bindings.jscSetInterval(this.handle, cb, ms); }
  setImmediate(cb: number): number { return this.bindings.jscSetImmediate(this.handle, cb); }
  clearTimeout(id: number): void { this.bindings.jscClearTimeout(this.handle, id); }
  clearInterval(id: number): void { this.bindings.jscClearInterval(this.handle, id); }
  clearImmediate(id: number): void { this.bindings.jscClearImmediate(this.handle, id); }
}

class JSCURLModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  parse(url: string): number { return this.bindings.jscURLParse(this.handle, url); }
  resolve(base: string, relative: string): string {
    const r = this.bindings.jscURLResolve(this.handle, base, relative);
    return r.data ?? "";
  }
  format(urlObj: number): string { return this.bindings.jscURLFormat(this.handle, urlObj); }
  domainToASCII(domain: string): string {
    const r = this.bindings.jscURLDomainToASCII(this.handle, domain);
    return r.data ?? "";
  }
}

class JSCFSModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  readFile(path: string): number { return this.bindings.jscFSReadFile(this.handle, path); }
  writeFile(path: string, data: number): void {
    const r = this.bindings.jscFSWriteFile(this.handle, path, data);
    if (!r.success) throw new Error("writeFile failed");
  }
  stat(path: string): any { return this.bindings.jscFSStat(this.handle, path); }
  lstat(path: string): any { return this.bindings.jscFSLstat(this.handle, path); }
  mkdir(path: string, mode?: number): void {
    const r = this.bindings.jscFSMkdir(this.handle, path, mode ?? 0o755);
    if (!r.success) throw new Error("mkdir failed");
  }
  mkdirp(path: string, mode?: number): void {
    const r = this.bindings.jscFSMkdirp(this.handle, path, mode ?? 0o755);
    if (!r.success) throw new Error("mkdirp failed");
  }
  readdir(path: string): string[] { return this.bindings.jscFSReaddir(this.handle, path); }
  unlink(path: string): void {
    const r = this.bindings.jscFSUnlink(this.handle, path);
    if (!r.success) throw new Error("unlink failed");
  }
  rmdir(path: string): void {
    const r = this.bindings.jscFSRmdir(this.handle, path);
    if (!r.success) throw new Error("rmdir failed");
  }
  rename(oldPath: string, newPath: string): void {
    const r = this.bindings.jscFSRename(this.handle, oldPath, newPath);
    if (!r.success) throw new Error("rename failed");
  }
  chmod(path: string, mode: number): void {
    const r = this.bindings.jscFSChmod(this.handle, path, mode);
    if (!r.success) throw new Error("chmod failed");
  }
  realpath(path: string): string { return this.bindings.jscFSRealpath(this.handle, path); }
  exists(path: string): boolean { return this.bindings.jscFSExists(this.handle, path); }
}

class JSCProcessModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  get pid(): number { return this.bindings.jscProcessPid(this.handle); }
  get ppid(): number { return this.bindings.jscProcessPpid(this.handle); }
  get cwd(): string { return this.bindings.jscProcessCwdStr(this.handle).data ?? ""; }
  get execPath(): string { return this.bindings.jscProcessExecPath(this.handle).data ?? ""; }
  get platform(): string { return this.bindings.jscProcessPlatform(this.handle); }
  get arch(): string { return this.bindings.jscProcessArch(this.handle); }
  get title(): string { return this.bindings.jscProcessTitle(this.handle).data ?? ""; }
  get argv(): string[] { return this.bindings.jscProcessArgv(this.handle); }
  get env(): Record<string, string> { return this.bindings.jscProcessEnvAll(this.handle); }
  exit(code?: number): void { this.bindings.jscProcessExit(this.handle, code ?? 0); }
  memoryUsage(): any { return this.bindings.jscProcessMemoryUsage(this.handle); }
  uptime(): number { return this.bindings.jscProcessUptime(this.handle); }
}

class JSCStreamModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  readableNew(options?: number): number { return this.bindings.jscStreamReadableNew(this.handle, options ?? 0); }
  writableNew(options?: number): number { return this.bindings.jscStreamWritableNew(this.handle, options ?? 0); }
  transformNew(options?: number): number { return this.bindings.jscStreamTransformNew(this.handle, options ?? 0); }
  push(stream: number, chunk: number): void {
    const r = this.bindings.jscStreamPush(this.handle, stream, chunk);
    if (!r.success) throw new Error("streamPush failed");
  }
  read(stream: number, size?: number): number { return this.bindings.jscStreamRead(this.handle, stream, size ?? 0); }
  end(stream: number): void {
    const r = this.bindings.jscStreamEnd(this.handle, stream);
    if (!r.success) throw new Error("streamEnd failed");
  }
  destroy(stream: number): void {
    const r = this.bindings.jscStreamDestroy(this.handle, stream);
    if (!r.success) throw new Error("streamDestroy failed");
  }
  pipe(readable: number, writable: number): number { return this.bindings.jscStreamPipe(this.handle, readable, writable); }
}

class JSCPathModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  basename(path: string): string { return this.bindings.jscPathBasename(this.handle, path).data ?? ""; }
  dirname(path: string): string { return this.bindings.jscPathDirname(this.handle, path).data ?? ""; }
  extname(path: string): string { return this.bindings.jscPathExtname(this.handle, path).data ?? ""; }
  join(...parts: string[]): string { return this.bindings.jscPathJoin(this.handle, parts).data ?? ""; }
  resolve(...parts: string[]): string { return this.bindings.jscPathResolve(this.handle, parts).data ?? ""; }
  normalize(path: string): string { return this.bindings.jscPathNormalize(this.handle, path).data ?? ""; }
  relative(from: string, to: string): string { return this.bindings.jscPathRelative(this.handle, from, to).data ?? ""; }
  isAbsolute(path: string): boolean { return this.bindings.jscPathIsAbsolute(this.handle, path); }
}

class JSCOperatingSystemModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
  hostname(): string { return this.bindings.jscOSHostname(this.handle); }
  platform(): string { return this.bindings.jscOSPlatform(this.handle); }
  arch(): string { return this.bindings.jscOSArch(this.handle); }
  release(): string { return this.bindings.jscOSRelease(this.handle); }
  type(): string { return this.bindings.jscOSType(this.handle); }
  uptime(): number { return this.bindings.jscOSUptime(this.handle); }
  totalMemory(): number { return this.bindings.jscOSTotalMemory(this.handle); }
  freeMemory(): number { return this.bindings.jscOSFreeMemory(this.handle); }
  cpus(): number { return this.bindings.jscOSCpus(this.handle); }
  loadAvg(): number[] { return this.bindings.jscOSLoadAvg(this.handle); }
  homedir(): string { return this.bindings.jscOSHomedir(this.handle); }
  tmpdir(): string { return this.bindings.jscOSTmpdir(this.handle); }
  userInfo(): { uid: number; gid: number; username: string; homedir: string; shell: string } {
    return this.bindings.jscOSUserInfo(this.handle);
  }
  info(): any { return this.bindings.jscOSInfo(this.handle); }
}

class JSCIsolateModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  enter(): void { this.bindings.jscIsolateEnter(this.handle); }
  exit(): void { this.bindings.jscIsolateExit(this.handle); }
}

class JSCContextModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  enter(): void { this.bindings.jscContextEnter(this.handle); }
  exit(): void { this.bindings.jscContextExit(this.handle); }
}

class JSCScriptModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  eval(code: string): string {
    const r = this.bindings.jscEval(this.handle, code);
    if (!r.success) throw new Error("eval failed");
    return r.data ?? "";
  }

  executeScript(filename: string, source: string): string {
    const r = this.bindings.jscExecuteScript(this.handle, filename, source);
    if (!r.success) throw new Error("executeScript failed");
    return r.data ?? "";
  }

  executeModule(filename: string, source: string): string {
    const r = this.bindings.jscExecuteModule(this.handle, filename, source);
    if (!r.success) throw new Error("executeModule failed");
    return r.data ?? "";
  }
}

class JSCJsonModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  stringify(valueHandle: number): string {
    const r = this.bindings.jscJsonStringify(this.handle, valueHandle);
    if (!r.success) throw new Error("JSON stringify failed");
    return r.data ?? "";
  }

  parse(json: string): number {
    return this.bindings.jscJsonParse(this.handle, json);
  }
}

class JSCGlobalModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  set(name: string, valueHandle: number): void {
    const r = this.bindings.jscSetGlobal(this.handle, name, valueHandle);
    if (!r.success) throw new Error("setGlobal failed");
  }

  get(name: string): number {
    return this.bindings.jscGetGlobal(this.handle, name);
  }
}

class JSCValueModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  newString(str: string): number { return this.bindings.jscValueNewString(this.handle, str); }
  newNumber(num: number): number { return this.bindings.jscValueNewNumber(this.handle, num); }
  newBool(val: boolean): number { return this.bindings.jscValueNewBool(this.handle, val); }
  newNull(): number { return this.bindings.jscValueNewNull(this.handle); }
  newUndefined(): number { return this.bindings.jscValueNewUndefined(this.handle); }
  newObject(): number { return this.bindings.jscValueNewObject(this.handle); }
  newArray(): number { return this.bindings.jscValueNewArray(this.handle); }

  typeOf(valueHandle: number): JSCValueType {
    return this.bindings.jscValueTypeof(this.handle, valueHandle).type;
  }

  toString(valueHandle: number): string {
    const r = this.bindings.jscValueToString(this.handle, valueHandle);
    if (!r.success) throw new Error("valueToString failed");
    return r.data ?? "";
  }

  toNumber(valueHandle: number): number {
    return this.bindings.jscValueToNumber(this.handle, valueHandle);
  }

  toBool(valueHandle: number): boolean {
    return this.bindings.jscValueToBool(this.handle, valueHandle);
  }

  isArray(valueHandle: number): boolean {
    return this.bindings.jscValueIsArray(this.handle, valueHandle);
  }

  dispose(valueHandle: number): void {
    this.bindings.jscValueDispose(valueHandle);
  }
}

class JSCPromiseModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  new(): number { return this.bindings.jscPromiseNew(this.handle); }

  resolve(promiseHandle: number, valueHandle: number): void {
    const r = this.bindings.jscPromiseResolve(this.handle, promiseHandle, valueHandle);
    if (!r.success) throw new Error("promiseResolve failed");
  }

  reject(promiseHandle: number, reason: string): void {
    const r = this.bindings.jscPromiseReject(this.handle, promiseHandle, reason);
    if (!r.success) throw new Error("promiseReject failed");
  }
}

class JSCModuleModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  compile(source: string, origin: string): number {
    return this.bindings.jscModuleCompile(this.handle, source, origin);
  }

  instantiate(moduleHandle: number): void {
    const r = this.bindings.jscModuleInstantiate(this.handle, moduleHandle);
    if (!r.success) throw new Error("moduleInstantiate failed");
  }

  evaluate(moduleHandle: number): string {
    const r = this.bindings.jscModuleEvaluate(this.handle, moduleHandle);
    if (!r.success) throw new Error("moduleEvaluate failed");
    return r.data ?? "";
  }

  dispose(moduleHandle: number): void {
    this.bindings.jscModuleDispose(moduleHandle);
  }
}

class JSCHeapModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  getStats(): JSCHeapStats {
    return this.bindings.jscGetHeapStats(this.handle);
  }

  requestGC(): void { this.bindings.jscRequestGC(this.handle); }
  lowMemoryNotification(): void { this.bindings.jscLowMemoryNotification(this.handle); }
}

class JSCErrorModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  getExceptionMessage(): string {
    return this.bindings.jscGetExceptionMessage(this.handle);
  }

  getStackTrace(): string {
    const r = this.bindings.jscGetStackTrace(this.handle);
    return r.data ?? "";
  }
}

class JSCMicrotaskModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}

  performCheck(): void { this.bindings.jscMicrotasksPerformCheck(this.handle); }
}

class JSCUtilsModule {
  constructor(private bindings: NativeJSCBindings) {}

  version(): string { return this.bindings.jscVersion(); }
}

class JSCInspectorModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
}

class JSCWasmModule {
  constructor(private handle: number, private bindings: NativeJSCBindings) {}
}
