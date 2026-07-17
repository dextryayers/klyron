export enum JSCValueType {
  Undefined = 0,
  Null = 1,
  Boolean = 2,
  Number = 3,
  String = 4,
  Object = 5,
  Array = 6,
  Function = 7,
  Error = 9,
  Symbol = 10,
  TypedArray = 13,
}

export interface JSCHeapStats {
  totalHeapSize: number;
  totalHeapSizeExecutable: number;
  totalPhysicalSize: number;
  totalAvailableSize: number;
  usedHeapSize: number;
  heapSizeLimit: number;
  mallocedMemory: number;
  peakMallocedMemory: number;
  numberOfNativeContexts: number;
  numberOfDetachedContexts: number;
  totalGlobalHandlesSize: number;
  usedGlobalHandlesSize: number;
  externalMemory: number;
}

export interface JSCResult {
  success: boolean;
}

export interface JSCStringResult {
  success: boolean;
  data: string | null;
}

export interface JSCOpaqueHandle {
  _handle: number;
}
