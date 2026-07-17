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
