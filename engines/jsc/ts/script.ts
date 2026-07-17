import type { NativeJSCBindings } from "./engine.ts";

export class JSCScript {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  eval(code: string): string {
    const r = this.bindings.jscEval(this.handle, code);
    if (!r.success) throw new Error("eval failed");
    return r.data ?? "";
  }

  executeScript(filename: string, source: string): string {
    const r = this.bindings.jscExecuteScript(this.handle, filename, source);
    if (!r.success) throw new Error("executeScript failed");
    return r.data ?? "";
  }

  executeModule(filename: string, source: string): string {
    const r = this.bindings.jscExecuteModule(this.handle, filename, source);
    if (!r.success) throw new Error("executeModule failed");
    return r.data ?? "";
  }
}
