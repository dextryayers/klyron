import type { NativeV8Bindings } from "./engine"

export class V8Console {
  private native: NativeV8Bindings
  private contextHandle: number
  private timestamps: boolean

  constructor(native: NativeV8Bindings, contextHandle: number, timestamps = false) {
    this.native = native
    this.contextHandle = contextHandle
    this.timestamps = timestamps
  }

  private formatMessage(args: unknown[]): string {
    return args.map(a => {
      if (a === null) return "null"
      if (a === undefined) return "undefined"
      if (typeof a === "object") {
        try { return JSON.stringify(a) } catch { return String(a) }
      }
      return String(a)
    }).join(" ")
  }

  private prefix(level: string): string {
    if (!this.timestamps) return `[${level}]`
    return `[${new Date().toISOString()}] [${level}]`
  }

  log(...args: unknown[]): void {
    this.native.consoleLog(this.contextHandle, `${this.prefix("LOG")} ${this.formatMessage(args)}`)
  }

  warn(...args: unknown[]): void {
    this.native.consoleWarn(this.contextHandle, `${this.prefix("WARN")} ${this.formatMessage(args)}`)
  }

  error(...args: unknown[]): void {
    this.native.consoleError(this.contextHandle, `${this.prefix("ERROR")} ${this.formatMessage(args)}`)
  }

  info(...args: unknown[]): void {
    this.native.consoleInfo(this.contextHandle, `${this.prefix("INFO")} ${this.formatMessage(args)}`)
  }

  debug(...args: unknown[]): void {
    this.native.consoleDebug(this.contextHandle, `${this.prefix("DEBUG")} ${this.formatMessage(args)}`)
  }

  table(data: unknown): void {
    if (!data || typeof data !== "object") {
      this.log(data)
      return
    }
    const rows: string[] = []
    if (Array.isArray(data)) {
      rows.push("(index)\t| value")
      rows.push("--------|-------")
      data.forEach((v, i) => rows.push(`${i}\t| ${JSON.stringify(v)}`))
    } else {
      rows.push("(key)\t| value")
      rows.push("--------|-------")
      for (const [k, v] of Object.entries(data)) {
        rows.push(`${k}\t| ${JSON.stringify(v)}`)
      }
    }
    this.log(rows.join("\n"))
  }

  time(label = "default"): void {
    const key = `__console_time_${label}`
    ;(globalThis as Record<string, number>)[key] = performance.now()
  }

  timeEnd(label = "default"): void {
    const key = `__console_time_${label}`
    const start = (globalThis as Record<string, number>)[key]
    if (start === undefined) {
      this.warn(`Timer '${label}' does not exist`)
      return
    }
    const duration = performance.now() - start
    this.log(`${label}: ${duration.toFixed(3)} ms`)
    delete (globalThis as Record<string, number>)[key]
  }

  count(label = "default"): void {
    const key = `__console_count_${label}`
    const val = ((globalThis as Record<string, number>)[key] ?? 0) + 1
    ;(globalThis as Record<string, number>)[key] = val
    this.log(`${label}: ${val}`)
  }

  countReset(label = "default"): void {
    const key = `__console_count_${label}`
    delete (globalThis as Record<string, number>)[key]
  }

  trace(...args: unknown[]): void {
    const err = new Error()
    const stack = err.stack?.split("\n").slice(2).join("\n") ?? ""
    this.log(`Trace: ${this.formatMessage(args)}\n${stack}`)
  }

  assert(condition: boolean, ...args: unknown[]): void {
    if (!condition) {
      this.error(`Assertion failed: ${this.formatMessage(args)}`)
    }
  }

  group(_label?: string): void {
  }

  groupEnd(): void {
  }

  clear(): void {
    this.native.consoleLog(this.contextHandle, "\n".repeat(50))
  }
}
