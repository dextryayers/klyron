import type { NativeJSCBindings } from "./engine.ts";

export class JSCTimers {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  setTimeout(callbackHandle: number, ms: number): number {
    return this.bindings.jscSetTimeout(this.handle, callbackHandle, ms);
  }

  setInterval(callbackHandle: number, ms: number): number {
    return this.bindings.jscSetInterval(this.handle, callbackHandle, ms);
  }

  setImmediate(callbackHandle: number): number {
    return this.bindings.jscSetImmediate(this.handle, callbackHandle);
  }

  clearTimeout(id: number): void {
    this.bindings.jscClearTimeout(this.handle, id);
  }

  clearInterval(id: number): void {
    this.bindings.jscClearInterval(this.handle, id);
  }

  clearImmediate(id: number): void {
    this.bindings.jscClearImmediate(this.handle, id);
  }
}
