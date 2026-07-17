import type { NativeV8Bindings } from "./engine"
import { V8Context } from "./context"

export class V8Snapshot {
  constructor(
    private native: NativeV8Bindings,
    private handle: number,
  ) {}

  static create(native: NativeV8Bindings, context: number): V8Snapshot {
    const handle = native.snapshotCreate(context)
    return new V8Snapshot(native, handle)
  }

  static load(native: NativeV8Bindings, blob: string, length: number): V8Snapshot {
    const handle = native.snapshotLoad(blob, length)
    return new V8Snapshot(native, handle)
  }

  get id(): number {
    return this.handle
  }

  dispose(): void {
    if (this.handle) {
      this.native.snapshotDispose(this.handle)
      this.handle = 0
    }
  }

  createContext(isolate: number): V8Context {
    const handle = this.native.contextNewFromSnapshot(isolate, this.handle)
    return new V8Context(this.native, handle)
  }
}
