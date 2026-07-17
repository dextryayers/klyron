export enum V8ValueType {
  Undefined = 0,
  Null = 1,
  Boolean = 2,
  Number = 3,
  String = 4,
  Object = 5,
  Array = 6,
  Function = 7,
  Promise = 8,
  Error = 9,
  Symbol = 10,
  BigInt = 11,
  Proxy = 12,
  TypedArray = 13,
}

export enum V8PromiseState {
  Pending = 0,
  Fulfilled = 1,
  Rejected = 2,
}

export enum V8MemoryPressure {
  None = 0,
  Moderate = 1,
  Critical = 2,
}

export interface V8HeapStats {
  totalHeapSize: number
  totalHeapSizeExecutable: number
  totalPhysicalSize: number
  totalAvailableSize: number
  usedHeapSize: number
  heapSizeLimit: number
  mallocedMemory: number
  peakMallocedMemory: number
  numberOfNativeContexts: number
  numberOfDetachedContexts: number
  totalGlobalHandlesSize: number
  usedGlobalHandlesSize: number
  externalMemory: number
}

export interface V8Config {
  icuDataPath?: string
  snapshotBlobPath?: string
  maxHeapSizeMb?: number
  initialHeapSizeMb?: number
  arrayBufferAllocatorPoolSize?: number
  useSharedMemory?: boolean
  exposeGc?: boolean
  singleThreaded?: boolean
}

export interface V8Result<T> {
  success: boolean
  data: T | null
  error: string | null
}

export interface V8StringResult {
  success: boolean
  data: string | null
  error: string | null
}

export interface V8TypeResult {
  type: V8ValueType
  success: boolean
  error: string | null
}

export interface V8ScriptMetadata {
  identity: number
  url: string
  lineOffset: number
  columnOffset: number
}

export interface V8StackFrame {
  functionName: string
  scriptName: string
  lineNumber: number
  columnNumber: number
}

export type V8HeapStatsResult = {
  success: boolean
  stats: V8HeapStats | null
  error: string | null
}
