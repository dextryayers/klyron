import type { NativeJSCBindings } from "./engine.ts";

export interface JSCRandomBytesResult {
  buffer: number;
  bytes: Uint8Array;
}

export class JSCCrypto {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  randomBytes(size: number): JSCRandomBytesResult {
    const buf = this.bindings.jscRandomBytes(this.handle, size);
    return { buffer: buf, bytes: new Uint8Array(size) };
  }

  randomFill(bufHandle: number, offset?: number, size?: number): number {
    return this.bindings.jscRandomFill(this.handle, bufHandle, offset ?? 0, size ?? 0);
  }

  randomUUID(): string {
    const r = this.bindings.jscRandomUUID(this.handle);
    return r;
  }
}
