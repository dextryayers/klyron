import type { NativeJSCBindings } from "./engine.ts";

export class JSCModule {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  compile(source: string, origin: string): number {
    return this.bindings.jscModuleCompile(this.handle, source, origin);
  }

  instantiate(moduleHandle: number): void {
    const r = this.bindings.jscModuleInstantiate(this.handle, moduleHandle);
    if (!r.success) throw new Error("moduleInstantiate failed");
  }

  evaluate(moduleHandle: number): string {
    const r = this.bindings.jscModuleEvaluate(this.handle, moduleHandle);
    if (!r.success) throw new Error("moduleEvaluate failed");
    return r.data ?? "";
  }

  dispose(moduleHandle: number): void {
    this.bindings.jscModuleDispose(moduleHandle);
  }
}
