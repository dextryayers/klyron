import type { NativeJSCBindings } from "./engine.ts";

export class JSCError {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  getMessage(): string {
    return this.bindings.jscGetExceptionMessage(this.handle);
  }

  getStackTrace(): string {
    const r = this.bindings.jscGetStackTrace(this.handle);
    return r.data ?? "";
  }
}
