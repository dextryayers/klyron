import type { NativeJSCBindings, JSCHeapStats } from "./engine.ts";
export type { JSCHeapStats };

export class JSCHeap {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  getStats(): JSCHeapStats {
    return this.bindings.jscGetHeapStats(this.handle);
  }

  requestGC(): void {
    this.bindings.jscRequestGC(this.handle);
  }

  lowMemoryNotification(): void {
    this.bindings.jscLowMemoryNotification(this.handle);
  }
}
