import type { NativeJSCBindings } from "./engine.ts";

export class JSCSnapshot {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  // JSC C API does not support snapshots — stubs only
}
