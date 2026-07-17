import type {
  V8Config,
  V8HeapStats,
  V8Result,
  V8StringResult,
  V8TypeResult,
  V8PromiseState,
  V8MemoryPressure,
} from "./types"
import { V8Isolate } from "./isolate"
import { V8Context } from "./context"
import { V8Script } from "./script"
import { V8Value } from "./value"
import { V8JSON } from "./json"
import { V8Global } from "./global"
import { V8Promise } from "./promise"
import { V8Module } from "./module"
import { V8Heap } from "./heap"
import { V8Snapshot } from "./snapshot"
import { V8Error } from "./error"
import { V8Microtask } from "./microtask"
import { V8Inspector } from "./inspector"
import { V8Wasm } from "./wasm"

export interface NativeV8Bindings {
  /* Platform lifecycle */
  init(config: V8Config | null): void
  shutdown(): void
  isInitialized(): boolean

  /* Version */
  version(): string
  majorVersion(): number
  minorVersion(): number
  buildVersion(): number
  patchVersion(): number

  /* Isolate */
  isolateNew(): number
  isolateDispose(handle: number): void
  isolateEnter(handle: number): void
  isolateExit(handle: number): void

  /* Context */
  contextNew(isolate: number): number
  contextNewFromSnapshot(isolate: number, snapshot: number): number
  contextDispose(handle: number): void
  contextEnter(handle: number): void
  contextExit(handle: number): void

  /* Script */
  compile(context: number, source: string, filename?: string): number
  run(context: number, script: number): V8StringResult
  eval(context: number, source: string, filename?: string): V8StringResult
  scriptDispose(handle: number): void

  /* JSON */
  jsonStringify(context: number, valueHandle: number): V8StringResult
  jsonParse(context: number, json: string): number

  /* Global */
  setGlobal(context: number, name: string, valueHandle: number): V8Result<null>
  getGlobal(context: number, name: string): number

  /* Value creation */
  valueNewString(context: number, str: string): number
  valueNewNumber(context: number, num: number): number
  valueNewBool(context: number, val: boolean): number
  valueNewNull(context: number): number
  valueNewUndefined(context: number): number
  valueNewObject(context: number): number
  valueNewArray(context: number): number

  /* Value inspection */
  valueTypeOf(context: number, valueHandle: number): V8TypeResult
  valueToString(context: number, valueHandle: number): V8StringResult
  valueToNumber(context: number, valueHandle: number): number
  valueToBool(context: number, valueHandle: number): boolean
  valueIsArray(context: number, valueHandle: number): boolean
  valueDispose(handle: number): void

  /* Promise */
  promiseNew(context: number): number
  promiseResolve(context: number, promise: number, valueHandle?: number): V8Result<null>
  promiseReject(context: number, promise: number, reason: string): V8Result<null>
  promiseGetNative(promise: number): number
  promiseState(context: number, promise: number): V8PromiseState
  promiseHasHandler(context: number, promise: number): boolean
  promiseMarkAsHandled(context: number, promise: number): V8Result<null>

  /* Microtasks */
  microtasksPerformCheck(context: number): void

  /* Module */
  moduleCompile(context: number, source: string, origin?: string): number
  moduleInstantiate(context: number, module: number): V8Result<null>
  moduleEvaluate(context: number, module: number): V8StringResult
  moduleGetIdentity(context: number, module: number): number
  moduleDispose(module: number): void

  /* Heap & memory */
  getHeapStats(isolate: number): { success: boolean; stats: V8HeapStats | null; error: string | null }
  lowMemoryNotification(isolate: number): void
  idleNotification(isolate: number, deadline: number): void
  setMemoryPressure(isolate: number, pressure: V8MemoryPressure): void
  requestGc(isolate: number): void
  getMallocedMemory(isolate: number): number
  adjustExternalMemory(isolate: number, change: number): number

  /* Snapshots */
  snapshotCreate(context: number): number
  snapshotLoad(blob: string, length: number): number
  snapshotDispose(handle: number): void

  /* Error */
  getException(context: number): number
  getExceptionMessage(context: number): string
  getStackTrace(context: number): V8StringResult

  /* Memory */
  freeString(s: string): void
  freeBuffer(buf: Uint8Array): void

  /* Inspector */
  inspectorNew(isolate: number): number
  inspectorDispose(id: number): void
  inspectorConnect(id: number, url: string): number
  inspectorDisconnect(session: number): void
  inspectorDispatch(session: number, message: string): string
  inspectorIsActive(): boolean

  /* WASM */
  wasmCompile(context: number, bytes: Uint8Array, length: number): number
  wasmInstantiate(context: number, bytes: Uint8Array, length: number, imports: number): number
}

export class V8Engine {
  private native: NativeV8Bindings
  private isolateHandle = 0
  private contextHandle = 0
  private _shutdown = false

  constructor(native: NativeV8Bindings) {
    this.native = native
  }

  initialize(config?: V8Config): void {
    this.native.init(config ?? null)
    this.isolateHandle = this.native.isolateNew()
    if (!this.isolateHandle) {
      throw new Error("V8Engine: failed to create isolate")
    }
    this.contextHandle = this.native.contextNew(this.isolateHandle)
    if (!this.contextHandle) {
      throw new Error("V8Engine: failed to create context")
    }
  }

  shutdown(): void {
    if (this._shutdown) return
    this._shutdown = true
    if (this.contextHandle) {
      this.native.contextDispose(this.contextHandle)
      this.contextHandle = 0
    }
    if (this.isolateHandle) {
      this.native.isolateDispose(this.isolateHandle)
      this.isolateHandle = 0
    }
    this.native.shutdown()
  }

  /* ─── version ─────────────────────────────────────────── */

  get version(): string {
    return this.native.version()
  }

  get majorVersion(): number {
    return this.native.majorVersion()
  }
  get minorVersion(): number {
    return this.native.minorVersion()
  }
  get buildVersion(): number {
    return this.native.buildVersion()
  }
  get patchVersion(): number {
    return this.native.patchVersion()
  }

  get isInitialized(): boolean {
    return this.native.isInitialized()
  }

  /* ─── sub-module accessors ────────────────────────────── */

  get isolate(): V8Isolate {
    return new V8Isolate(this.native, this.isolateHandle)
  }

  get context(): V8Context {
    return new V8Context(this.native, this.contextHandle)
  }

  get json(): V8JSON {
    return new V8JSON(this.native, this.contextHandle)
  }

  get global(): V8Global {
    return new V8Global(this.native, this.contextHandle)
  }

  get heap(): V8Heap {
    return new V8Heap(this.native, this.isolateHandle)
  }

  get error(): V8Error {
    return new V8Error(this.native, this.contextHandle)
  }

  get microtask(): V8Microtask {
    return new V8Microtask(this.native, this.contextHandle)
  }

  get inspector(): V8Inspector {
    return new V8Inspector(this.native, this.contextHandle)
  }

  get wasm(): V8Wasm {
    return new V8Wasm(this.native, this.contextHandle)
  }

  /* ─── eval / script ───────────────────────────────────── */

  eval(source: string, filename?: string): V8StringResult {
    return this.native.eval(this.contextHandle, source, filename)
  }

  compileScript(source: string, filename?: string): V8Script {
    return V8Script.compile(this.native, this.contextHandle, source, filename)
  }

  /* ─── promise ─────────────────────────────────────────── */

  createPromise(): V8Promise {
    return V8Promise.create(this.native, this.contextHandle)
  }

  /* ─── module ──────────────────────────────────────────── */

  compileModule(source: string, origin?: string): V8Module {
    return V8Module.compile(this.native, this.contextHandle, source, origin)
  }

  /* ─── snapshot ────────────────────────────────────────── */

  createSnapshot(): V8Snapshot {
    return V8Snapshot.create(this.native, this.contextHandle)
  }
}
