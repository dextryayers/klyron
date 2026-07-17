import type { NativeJSCBindings } from "./engine.ts";

export class JSCJson {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  stringify(valueHandle: number): string {
    const r = this.bindings.jscJsonStringify(this.handle, valueHandle);
    if (!r.success) throw new Error("JSON stringify failed");
    return r.data ?? "";
  }

  parse(json: string): number {
    return this.bindings.jscJsonParse(this.handle, json);
  }
}
