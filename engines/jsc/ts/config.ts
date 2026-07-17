export interface JSCCreateOptions {
  maxHeapSizeMB?: number;
  exposeGC?: boolean;
}

export const DEFAULT_JSC_CONFIG: JSCCreateOptions = {};
