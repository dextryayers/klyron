import type { NativeJSCBindings } from "./engine.ts";

export class JSCIsolate {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  enter(): void {
    this.bindings.jscIsolateEnter(this.handle);
  }

  exit(): void {
    this.bindings.jscIsolateExit(this.handle);
  }
}
