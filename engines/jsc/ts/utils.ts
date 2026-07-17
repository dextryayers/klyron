import type { NativeJSCBindings } from "./engine.ts";

export class JSCUtils {
  constructor(private bindings: NativeJSCBindings) {}

  version(): string {
    return this.bindings.jscVersion();
  }
}
