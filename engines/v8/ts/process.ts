import type { NativeV8Bindings } from "./engine"

export interface ProcessInfo {
  pid: number
  ppid: number
  cwd: string
  execPath: string
  platform: string
  title: string
  argv: string[]
  env: Record<string, string>
}

export class V8Process {
  private native: NativeV8Bindings
  private contextHandle: number

  constructor(native: NativeV8Bindings, contextHandle: number) {
    this.native = native
    this.contextHandle = contextHandle
  }

  get pid(): number {
    return this.native.processGetPid(this.contextHandle)
  }

  get ppid(): number {
    return this.native.processGetPpid(this.contextHandle)
  }

  get cwd(): string {
    return this.native.processGetCwd(this.contextHandle) ?? "/"
  }

  get execPath(): string {
    return this.native.processGetExecPath(this.contextHandle) ?? ""
  }

  get platform(): string {
    return this.native.processGetPlatform(this.contextHandle) ?? "linux"
  }

  get title(): string {
    return this.native.processGetTitle(this.contextHandle) ?? ""
  }

  get argv(): string[] {
    return this.native.processGetArgv(this.contextHandle) ?? []
  }

  get env(): Record<string, string> {
    return this.native.processGetEnv(this.contextHandle) ?? {}
  }

  envGet(name: string): string | undefined {
    return this.native.processEnvGet(this.contextHandle, name) ?? undefined
  }

  exit(code = 0): never {
    this.native.processExit(this.contextHandle, code)
    throw new Error("exit")
  }

  uptime(): number {
    return process.uptime()
  }

  memoryUsage(): { rss: number; heapTotal: number; heapUsed: number; external: number } {
    return process.memoryUsage()
  }

  hrtime(): [number, number] {
    const t = Date.now() * 1e6
    return [Math.floor(t / 1e9), Math.floor(t % 1e9)]
  }

  getProcessInfo(): ProcessInfo {
    return {
      pid: this.pid,
      ppid: this.ppid,
      cwd: this.cwd,
      execPath: this.execPath,
      platform: this.platform,
      title: this.title,
      argv: this.argv,
      env: this.env,
    }
  }
}
