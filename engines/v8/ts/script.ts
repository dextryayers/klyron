import type { NativeV8Bindings } from "./engine"
import type { V8StringResult } from "./types"

export class V8Script {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
    private handle: number,
  ) {}

  static compile(
    native: NativeV8Bindings,
    context: number,
    source: string,
    filename?: string,
  ): V8Script {
    const handle = native.compile(context, source, filename)
    return new V8Script(native, context, handle)
  }

  get id(): number {
    return this.handle
  }

  run(): V8StringResult {
    return this.native.run(this.context, this.handle)
  }

  dispose(): void {
    if (this.handle) {
      this.native.scriptDispose(this.handle)
      this.handle = 0
    }
  }
}
