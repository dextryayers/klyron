import type { NativeV8Bindings } from "./engine"
import type { V8StackFrame } from "./types"
import { V8Value } from "./value"

export class V8Error {
  constructor(
    private native: NativeV8Bindings,
    private context: number,
  ) {}

  getException(): V8Value {
    const handle = this.native.getException(this.context)
    return V8Value.wrap(this.native, this.context, handle)
  }

  getExceptionMessage(): string {
    return this.native.getExceptionMessage(this.context)
  }

  getStackTrace(): string | null {
    const result = this.native.getStackTrace(this.context)
    return result.data
  }

  parseStackTrace(): V8StackFrame[] {
    const stack = this.getStackTrace()
    if (!stack) return []

    const frames: V8StackFrame[] = []
    const lines = stack.split("\n")
    for (const line of lines) {
      const trimmed = line.trim()
      if (!trimmed || trimmed.startsWith("Error")) continue

      const match = trimmed.match(
        /at\s+(?:(.+?)\s+\()?(?:(.+?):(\d+):(\d+)\)?)?/,
      )
      if (match) {
        frames.push({
          functionName: match[1] || "<anonymous>",
          scriptName: match[2] || "<unknown>",
          lineNumber: parseInt(match[3], 10) || 0,
          columnNumber: parseInt(match[4], 10) || 0,
        })
      }
    }
    return frames
  }
}
