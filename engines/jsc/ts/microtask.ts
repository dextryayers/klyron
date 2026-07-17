import type { NativeJSCBindings } from "./engine.ts";

export class JSCMicrotask {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  performCheck(): void {
    this.bindings.jscMicrotasksPerformCheck(this.handle);
  }
}
