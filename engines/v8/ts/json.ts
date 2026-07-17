import type { NativeV8Bindings } from "./engine"
import type { V8StringResult } from "./types"
import { V8Value } from "./value"

export class V8JSON {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
  ) {}

  stringify(valueHandle: number): V8StringResult {
    return this.native.jsonStringify(this.context, valueHandle)
  }

  stringifyValue(value: V8Value): V8StringResult {
    return this.stringify(value.id)
  }

  parse(json: string): V8Value {
    const handle = this.native.jsonParse(this.context, json)
    return V8Value.wrap(this.native, this.context, handle)
  }
}
