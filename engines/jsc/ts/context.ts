import type { NativeJSCBindings } from "./engine.ts";

export class JSCContext {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  enter(): void {
    this.bindings.jscContextEnter(this.handle);
  }

  exit(): void {
    this.bindings.jscContextExit(this.handle);
  }
}
