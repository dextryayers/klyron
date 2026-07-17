import type { NativeJSCBindings } from "./engine.ts";

export class JSCInspector {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  // JSC C API does not expose inspector protocol — stubs only
}
