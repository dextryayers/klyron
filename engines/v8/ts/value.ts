import type { NativeV8Bindings } from "./engine"
import type { V8StringResult, V8ValueType } from "./types"

export class V8Value {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
    private handle: number = 0,
  ) {}

  static wrap(native: NativeV8Bindings, context: number, handle: number): V8Value {
    return new V8Value(native, context, handle)
  }

  static createString(native: NativeV8Bindings, context: number, str: string): V8Value {
    const handle = native.valueNewString(context, str)
    return new V8Value(native, context, handle)
  }

  static createNumber(native: NativeV8Bindings, context: number, num: number): V8Value {
    const handle = native.valueNewNumber(context, num)
    return new V8Value(native, context, handle)
  }

  static createBoolean(native: NativeV8Bindings, context: number, val: boolean): V8Value {
    const handle = native.valueNewBool(context, val)
    return new V8Value(native, context, handle)
  }

  static createNull(native: NativeV8Bindings, context: number): V8Value {
    const handle = native.valueNewNull(context)
    return new V8Value(native, context, handle)
  }

  static createUndefined(native: NativeV8Bindings, context: number): V8Value {
    const handle = native.valueNewUndefined(context)
    return new V8Value(native, context, handle)
  }

  static createObject(native: NativeV8Bindings, context: number): V8Value {
    const handle = native.valueNewObject(context)
    return new V8Value(native, context, handle)
  }

  static createArray(native: NativeV8Bindings, context: number): V8Value {
    const handle = native.valueNewArray(context)
    return new V8Value(native, context, handle)
  }

  get id(): number {
    return this.handle
  }

  typeOf(): V8ValueType {
    const result = this.native.valueTypeOf(this.context, this.handle)
    return result.type
  }

  toString(): V8StringResult {
    return this.native.valueToString(this.context, this.handle)
  }

  toNumber(): number {
    return this.native.valueToNumber(this.context, this.handle)
  }

  toBool(): boolean {
    return this.native.valueToBool(this.context, this.handle)
  }

  isArray(): boolean {
    return this.native.valueIsArray(this.context, this.handle)
  }

  dispose(): void {
    if (this.handle) {
      this.native.valueDispose(this.handle)
      this.handle = 0
    }
  }
}
