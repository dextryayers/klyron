import type { NativeJSCBindings, JSCValueType } from "./engine.ts";
export type { JSCValueType };

export class JSCValue {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  newString(str: string): number {
    return this.bindings.jscValueNewString(this.handle, str);
  }

  newNumber(num: number): number {
    return this.bindings.jscValueNewNumber(this.handle, num);
  }

  newBool(val: boolean): number {
    return this.bindings.jscValueNewBool(this.handle, val);
  }

  newNull(): number {
    return this.bindings.jscValueNewNull(this.handle);
  }

  newUndefined(): number {
    return this.bindings.jscValueNewUndefined(this.handle);
  }

  newObject(): number {
    return this.bindings.jscValueNewObject(this.handle);
  }

  newArray(): number {
    return this.bindings.jscValueNewArray(this.handle);
  }

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
