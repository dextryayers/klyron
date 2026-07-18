import type { NativeV8Bindings } from "./engine"

export interface CPUInfo {
  model: string
  speed: number
  times: {
    user: number
    nice: number
    sys: number
    idle: number
    irq: number
  }
}

export interface NetworkInterfaceInfo {
  address: string
  netmask: string
  family: string
  mac: string
  internal: boolean
  cidr: string | null
}

export class V8OS {
  private native: NativeV8Bindings
  private contextHandle: number

  constructor(native: NativeV8Bindings, contextHandle: number) {
    this.native = native
    this.contextHandle = contextHandle
  }

  arch(): string {
    return typeof process !== "undefined" ? process.arch : "x64"
  }

  platform(): string {
    return typeof process !== "undefined" ? process.platform : "linux"
  }

  release(): string {
    return typeof process !== "undefined" ? process.release?.name ?? "linux" : "linux"
  }

  type(): string {
    return "Linux"
  }

  version(): string {
    return ""
  }

  homedir(): string {
    return this.native.osHomedir(this.contextHandle) ?? "/home/user"
  }

  tmpdir(): string {
    return "/tmp"
  }

  hostname(): string {
    return this.native.osHostname(this.contextHandle) ?? "localhost"
  }

  endianness(): "BE" | "LE" {
    const buf = new ArrayBuffer(2)
    const view = new DataView(buf)
    view.setInt16(0, 256, true)
    return view.getInt16(0, true) === 256 ? "LE" : "BE"
  }

  loadavg(): number[] {
    return [0, 0, 0]
  }

  uptime(): number {
    return this.native.osUptime(this.contextHandle) ?? 0
  }

  totalmem(): number {
    return this.native.osTotalMemory(this.contextHandle) ?? 0
  }

  freemem(): number {
    return this.native.osFreeMemory(this.contextHandle) ?? 0
  }

  cpus(): CPUInfo[] {
    const arr = this.native.osCpus(this.contextHandle) as CPUInfo[] ?? []
    return arr
  }

  networkInterfaces(): Record<string, NetworkInterfaceInfo[]> {
    return this.native.osNetworkInterfaces(this.contextHandle) ?? {}
  }

  userInfo(): { username: string; uid: number; gid: number; shell: string; homedir: string } {
    const info = this.native.osUserInfo(this.contextHandle) as Record<string, unknown> ?? {}
    return {
      username: String(info.username ?? "user"),
      uid: Number(info.uid ?? 1000),
      gid: Number(info.gid ?? 1000),
      shell: String(info.shell ?? "/bin/bash"),
      homedir: String(info.homedir ?? "/home/user"),
    }
  }

  EOL(): string {
    return "\n"
  }

  priority(pid?: number): number {
    return this.native.osGetPriority(this.contextHandle, pid ?? 0) ?? 0
  }

  setPriority(pid: number, priority: number): void {
    this.native.osSetPriority(this.contextHandle, pid, priority)
  }
}
