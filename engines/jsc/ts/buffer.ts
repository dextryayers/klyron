import type { NativeJSCBindings } from "./engine.ts";

export class JSCBuffer {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  alloc(size: number): number {
    return this.bindings.jscBufferAlloc(this.handle, size);
  }

  fromString(str: string): number {
    return this.bindings.jscBufferFromString(this.handle, str);
  }

  concat(bufHandles: number[]): number {
    return this.bindings.jscBufferConcat(this.handle, bufHandles);
  }

  toString(bufHandle: number, encoding?: string): string {
    const r = this.bindings.jscBufferToString(this.handle, bufHandle, encoding ?? "utf8");
    if (!r.success) throw new Error("bufferToString failed");
    return r.data ?? "";
  }

  slice(bufHandle: number, start: number, end: number): number {
    return this.bindings.jscBufferSlice(this.handle, bufHandle, start, end);
  }
}
