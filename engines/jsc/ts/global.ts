import type { NativeJSCBindings } from "./engine.ts";

export class JSCGlobal {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  set(name: string, valueHandle: number): void {
    const r = this.bindings.jscSetGlobal(this.handle, name, valueHandle);
    if (!r.success) throw new Error("setGlobal failed");
  }

  get(name: string): number {
    return this.bindings.jscGetGlobal(this.handle, name);
  }
}
