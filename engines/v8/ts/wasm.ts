import type { NativeV8Bindings } from "./engine"
import type { V8StringResult } from "./types"

export class V8Wasm {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
  ) {}

  compile(bytes: Uint8Array): V8StringResult {
    const values = Array.from(bytes)
      .map(b => b.toString())
      .join(",")
    const source = `new WebAssembly.Module(new Uint8Array([${values}]))`
    return this.native.eval(this.context, source, "wasm:compile")
  }
}
