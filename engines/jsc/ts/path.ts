import type { NativeJSCBindings } from "./engine.ts";

export class JSCPath {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  basename(path: string): string {
    const r = this.bindings.jscPathBasename(this.handle, path);
    if (!r.success) throw new Error("pathBasename failed");
    return r.data ?? "";
  }

  dirname(path: string): string {
    const r = this.bindings.jscPathDirname(this.handle, path);
    if (!r.success) throw new Error("pathDirname failed");
    return r.data ?? "";
  }

  extname(path: string): string {
    const r = this.bindings.jscPathExtname(this.handle, path);
    if (!r.success) throw new Error("pathExtname failed");
    return r.data ?? "";
  }

  join(...parts: string[]): string {
    const r = this.bindings.jscPathJoin(this.handle, parts);
    if (!r.success) throw new Error("pathJoin failed");
    return r.data ?? "";
  }

  resolve(...parts: string[]): string {
    const r = this.bindings.jscPathResolve(this.handle, parts);
    if (!r.success) throw new Error("pathResolve failed");
    return r.data ?? "";
  }

  normalize(path: string): string {
    const r = this.bindings.jscPathNormalize(this.handle, path);
    if (!r.success) throw new Error("pathNormalize failed");
    return r.data ?? "";
  }

  relative(from: string, to: string): string {
    const r = this.bindings.jscPathRelative(this.handle, from, to);
    if (!r.success) throw new Error("pathRelative failed");
    return r.data ?? "";
  }

  isAbsolute(path: string): boolean {
    return this.bindings.jscPathIsAbsolute(this.handle, path);
  }
}
