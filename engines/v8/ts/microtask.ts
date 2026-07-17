import type { NativeV8Bindings } from "./engine"

export class V8Microtask {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
  ) {}

  performCheckpoint(): void {
    this.native.microtasksPerformCheck(this.context)
  }
}
