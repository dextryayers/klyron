import type { NativeV8Bindings } from "./engine"
import { V8Snapshot } from "./snapshot"

export class V8Context {
  constructor(
    private native: NativeV8Bindings,
    private handle: number,
  ) {}

  static create(native: NativeV8Bindings, isolate: number): V8Context {
    const handle = native.contextNew(isolate)
    return new V8Context(native, handle)
  }

  static fromSnapshot(
    native: NativeV8Bindings,
    isolate: number,
    snapshot: V8Snapshot,
  ): V8Context {
    const handle = native.contextNewFromSnapshot(isolate, snapshot.id)
    return new V8Context(native, handle)
  }

  get id(): number {
    return this.handle
  }

  enter(): void {
    this.native.contextEnter(this.handle)
  }

  exit(): void {
    this.native.contextExit(this.handle)
  }

  dispose(): void {
    if (this.handle) {
      this.native.contextDispose(this.handle)
      this.handle = 0
    }
  }
}
