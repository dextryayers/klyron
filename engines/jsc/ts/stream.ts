import type { NativeJSCBindings } from "./engine.ts";

export class JSCStream {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  readableNew(optionsHandle?: number): number {
    return this.bindings.jscStreamReadableNew(this.handle, optionsHandle ?? 0);
  }

  writableNew(optionsHandle?: number): number {
    return this.bindings.jscStreamWritableNew(this.handle, optionsHandle ?? 0);
  }

  transformNew(optionsHandle?: number): number {
    return this.bindings.jscStreamTransformNew(this.handle, optionsHandle ?? 0);
  }

  push(streamHandle: number, chunkHandle: number): void {
    const r = this.bindings.jscStreamPush(this.handle, streamHandle, chunkHandle);
    if (!r.success) throw new Error("streamPush failed");
  }

  read(streamHandle: number, size?: number): number {
    return this.bindings.jscStreamRead(this.handle, streamHandle, size ?? 0);
  }

  end(streamHandle: number): void {
    const r = this.bindings.jscStreamEnd(this.handle, streamHandle);
    if (!r.success) throw new Error("streamEnd failed");
  }

  destroy(streamHandle: number): void {
    const r = this.bindings.jscStreamDestroy(this.handle, streamHandle);
    if (!r.success) throw new Error("streamDestroy failed");
  }

  pipe(readableHandle: number, writableHandle: number): number {
    return this.bindings.jscStreamPipe(this.handle, readableHandle, writableHandle);
  }
}
