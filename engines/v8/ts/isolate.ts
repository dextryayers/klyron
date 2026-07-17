import type { NativeV8Bindings } from "./engine"

export class V8Isolate {
  constructor(
    private native: NativeV8Bindings,
    private handle: number,
  ) {}

  get id(): number {
    return this.handle
  }

  enter(): void {
    this.native.isolateEnter(this.handle)
  }

  exit(): void {
    this.native.isolateExit(this.handle)
  }

  dispose(): void {
    if (this.handle) {
      this.native.isolateDispose(this.handle)
      this.handle = 0
    }
  }
}
