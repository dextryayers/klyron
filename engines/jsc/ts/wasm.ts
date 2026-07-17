import type { NativeJSCBindings } from "./engine.ts";

export class JSCWasm {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  // JSC C API does not expose native Wasm compilation.
  // WebAssembly.compile/instantiate are available via jscEval().
}
