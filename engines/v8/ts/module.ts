import type { NativeV8Bindings } from "./engine"
import type { V8Result, V8StringResult } from "./types"

export class V8Module {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
    private handle: number,
  ) {}

  static compile(
    native: NativeV8Bindings,
    context: number,
    source: string,
    origin?: string,
  ): V8Module {
    const handle = native.moduleCompile(context, source, origin)
    return new V8Module(native, context, handle)
  }

  get id(): number {
    return this.handle
  }

  instantiate(): V8Result<null> {
    return this.native.moduleInstantiate(this.context, this.handle)
  }

  evaluate(): V8StringResult {
    return this.native.moduleEvaluate(this.context, this.handle)
  }

  get identity(): number {
    return this.native.moduleGetIdentity(this.context, this.handle)
  }

  dispose(): void {
    if (this.handle) {
      this.native.moduleDispose(this.handle)
      this.handle = 0
    }
  }
}
