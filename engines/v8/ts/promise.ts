import type { NativeV8Bindings } from "./engine"
import type { V8PromiseState, V8Result } from "./types"
import { V8Value } from "./value"

export class V8Promise {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
    private handle: number,
  ) {}

  static create(native: NativeV8Bindings, context: number): V8Promise {
    const handle = native.promiseNew(context)
    return new V8Promise(native, context, handle)
  }

  static wrap(native: NativeV8Bindings, context: number, handle: number): V8Promise {
    return new V8Promise(native, context, handle)
  }

  get id(): number {
    return this.handle
  }

  resolve(valueHandle?: number): V8Result<null> {
    return this.native.promiseResolve(this.context, this.handle, valueHandle)
  }

  reject(reason: string): V8Result<null> {
    return this.native.promiseReject(this.context, this.handle, reason)
  }

  get state(): V8PromiseState {
    return this.native.promiseState(this.context, this.handle)
  }

  hasHandler(): boolean {
    return this.native.promiseHasHandler(this.context, this.handle)
  }

  markAsHandled(): V8Result<null> {
    return this.native.promiseMarkAsHandled(this.context, this.handle)
  }

  getNative(): V8Value {
    const valueHandle = this.native.promiseGetNative(this.handle)
    return V8Value.wrap(this.native, this.context, valueHandle)
  }
}
