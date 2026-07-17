import type { NativeV8Bindings } from "./engine"
import type { V8Result } from "./types"
import { V8Value } from "./value"

export class V8Global {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
  ) {}

  set(name: string, valueHandle: number): V8Result<null> {
    return this.native.setGlobal(this.context, name, valueHandle)
  }

  setValue(name: string, value: V8Value): V8Result<null> {
    return this.set(name, value.id)
  }

  get(name: string): V8Value {
    const handle = this.native.getGlobal(this.context, name)
    return V8Value.wrap(this.native, this.context, handle)
  }
}
