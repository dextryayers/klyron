import type { NativeJSCBindings } from "./engine.ts";

export class JSCEncoding {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  encodeText(input: string): string {
    const r = this.bindings.jscEncodeText(this.handle, input);
    if (!r.success) throw new Error("encodeText failed");
    return r.data ?? "";
  }

  encodeInto(input: string, dstHandle: number, dstOffset: number): number {
    return this.bindings.jscEncodeInto(this.handle, input, dstHandle, dstOffset);
  }

  decodeText(bufHandle: number, encoding?: string): string {
    const r = this.bindings.jscDecodeText(this.handle, bufHandle, encoding ?? "utf8");
    return r;
  }

  base64Encode(input: string): string {
    const r = this.bindings.jscBase64Encode(this.handle, input);
    if (!r.success) throw new Error("base64Encode failed");
    return r.data ?? "";
  }

  base64Decode(input: string): number {
    return this.bindings.jscBase64Decode(this.handle, input);
  }

  hexEncode(input: string): string {
    const r = this.bindings.jscHexEncode(this.handle, input);
    if (!r.success) throw new Error("hexEncode failed");
    return r.data ?? "";
  }

  hexDecode(input: string): number {
    return this.bindings.jscHexDecode(this.handle, input);
  }
}
