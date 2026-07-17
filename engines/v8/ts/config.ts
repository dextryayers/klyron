import type { V8Config } from "./types"

export class V8ConfigBuilder {
  private config: V8Config = {}

  icuDataPath(path: string): this {
    this.config.icuDataPath = path
    return this
  }

  snapshotBlobPath(path: string): this {
    this.config.snapshotBlobPath = path
    return this
  }

  maxHeapSizeMb(size: number): this {
    this.config.maxHeapSizeMb = size
    return this
  }

  initialHeapSizeMb(size: number): this {
    this.config.initialHeapSizeMb = size
    return this
  }

  arrayBufferAllocatorPoolSize(size: number): this {
    this.config.arrayBufferAllocatorPoolSize = size
    return this
  }

  useSharedMemory(use: boolean): this {
    this.config.useSharedMemory = use
    return this
  }

  exposeGc(expose: boolean): this {
    this.config.exposeGc = expose
    return this
  }

  singleThreaded(single: boolean): this {
    this.config.singleThreaded = single
    return this
  }

  build(): V8Config {
    return { ...this.config }
  }
}
