import type { NativeJSCBindings } from "./engine.ts";

export class JSCPromise {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  new(): number {
    return this.bindings.jscPromiseNew(this.handle);
  }

  resolve(promiseHandle: number, valueHandle: number): void {
    const r = this.bindings.jscPromiseResolve(this.handle, promiseHandle, valueHandle);
    if (!r.success) throw new Error("promiseResolve failed");
  }

  reject(promiseHandle: number, reason: string): void {
    const r = this.bindings.jscPromiseReject(this.handle, promiseHandle, reason);
    if (!r.success) throw new Error("promiseReject failed");
  }
}
