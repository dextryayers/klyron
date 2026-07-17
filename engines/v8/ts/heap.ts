import type { NativeV8Bindings } from "./engine"
import type { V8HeapStats, V8MemoryPressure } from "./types"

export class V8Heap {
  constructor(
    private native: NativeV8Bindings,
    private isolate: number,
  ) {}

  getStats(): V8HeapStats | null {
    const result = this.native.getHeapStats(this.isolate)
    return result.stats ?? null
  }

  lowMemoryNotification(): void {
    this.native.lowMemoryNotification(this.isolate)
  }

  idleNotification(deadlineInSeconds: number): void {
    this.native.idleNotification(this.isolate, deadlineInSeconds)
  }

  setMemoryPressure(pressure: V8MemoryPressure): void {
    this.native.setMemoryPressure(this.isolate, pressure)
  }

  requestGc(): void {
    this.native.requestGc(this.isolate)
  }

  getMallocedMemory(): number {
    return this.native.getMallocedMemory(this.isolate)
  }

  adjustExternalMemory(change: number): number {
    return this.native.adjustExternalMemory(this.isolate, change)
  }
}
