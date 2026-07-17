import type { NativeV8Bindings } from "./engine"

export class V8Inspector {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
  ) {}
}
