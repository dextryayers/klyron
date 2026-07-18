import type { NativeV8Bindings } from "./engine"

export interface FSStat {
  dev: number
  ino: number
  mode: number
  uid: number
  gid: number
  size: number
  blksize: number
  blocks: number
  atime: number
  mtime: number
  ctime: number
  type: "file" | "dir" | "symlink" | "other"
}

export class V8FS {
  private native: NativeV8Bindings
  private contextHandle: number

  constructor(native: NativeV8Bindings, contextHandle: number) {
    this.native = native
    this.contextHandle = contextHandle
  }

  readFileSync(path: string): Uint8Array {
    const result = this.native.fsReadFile(this.contextHandle, path)
    if (!result.success) throw new Error(result.error ?? "readFile failed")
    const handle = result.data as unknown as number
    const len = this.native.bufferGetLength(this.contextHandle, handle)
    const arr = new Uint8Array(len)
    for (let i = 0; i < len; i++) {
      arr[i] = this.native.getValueAtIndex(this.contextHandle, handle, i)
    }
    return arr
  }

  writeFileSync(path: string, data: string | Uint8Array): void {
    let bytes: Uint8Array
    if (typeof data === "string") {
      bytes = new TextEncoder().encode(data)
    } else {
      bytes = data
    }
    const result = this.native.fsWriteFile(this.contextHandle, path, bytes, bytes.length)
    if (!result.success) throw new Error(result.error ?? "writeFile failed")
  }

  appendFileSync(path: string, data: string | Uint8Array): void {
    let bytes: Uint8Array
    if (typeof data === "string") {
      bytes = new TextEncoder().encode(data)
    } else {
      bytes = data
    }
    const result = this.native.fsAppendFile(this.contextHandle, path, bytes, bytes.length)
    if (!result.success) throw new Error(result.error ?? "appendFile failed")
  }

  mkdirSync(path: string, options?: { mode?: number; recursive?: boolean }): void {
    if (options?.recursive) {
      const parts = path.split("/").filter(Boolean)
      let acc = ""
      for (const part of parts) {
        acc += "/" + part
        const stat = this.statSyncNoThrow(acc)
        if (!stat) {
          const result = this.native.fsMkdir(this.contextHandle, acc, options.mode ?? 0o755)
          if (!result.success) throw new Error(result.error ?? "mkdir failed")
        } else if (stat.type !== "dir") {
          throw new Error(`${acc} is not a directory`)
        }
      }
    } else {
      const result = this.native.fsMkdir(this.contextHandle, path, options?.mode ?? 0o755)
      if (!result.success) throw new Error(result.error ?? "mkdir failed")
    }
  }

  rmdirSync(path: string): void {
    const result = this.native.fsRmdir(this.contextHandle, path)
    if (!result.success) throw new Error(result.error ?? "rmdir failed")
  }

  unlinkSync(path: string): void {
    const result = this.native.fsUnlink(this.contextHandle, path)
    if (!result.success) throw new Error(result.error ?? "unlink failed")
  }

  renameSync(oldPath: string, newPath: string): void {
    const result = this.native.fsRename(this.contextHandle, oldPath, newPath)
    if (!result.success) throw new Error(result.error ?? "rename failed")
  }

  existsSync(path: string): boolean {
    const result = this.native.fsExists(this.contextHandle, path)
    if (!result.success) return false
    return !!(result as unknown as { exists: boolean }).exists
  }

  statSync(path: string): FSStat {
    const result = this.native.fsStat(this.contextHandle, path)
    if (!result.success) throw new Error(result.error ?? "stat failed")
    return this.toFSStat(result)
  }

  private statSyncNoThrow(path: string): FSStat | null {
    try {
      return this.statSync(path)
    } catch {
      return null
    }
  }

  private toFSStat(result: unknown): FSStat {
    const r = result as Record<string, unknown>
    const typeMap = ["file", "dir", "symlink", "other"]
    const t = (r.type as number) ?? 3
    return {
      dev: r.dev as number,
      ino: r.ino as number,
      mode: r.mode as number,
      uid: r.uid as number,
      gid: r.gid as number,
      size: r.size as number,
      blksize: r.blksize as number,
      blocks: r.blocks as number,
      atime: r.atime as number,
      mtime: r.mtime as number,
      ctime: r.ctime as number,
      type: typeMap[t] as FSStat["type"],
    }
  }

  readdirSync(path: string): string[] {
    const handle = this.native.fsReaddir(this.contextHandle, path)
    if (!handle) throw new Error(`readdir failed: ${path}`)
    const len = this.native.getValueLength(this.contextHandle, handle) as number
    const entries: string[] = []
    for (let i = 0; i < len; i++) {
      const val = this.native.getValueAtIndex(this.contextHandle, handle, i)
      entries.push(String(val))
    }
    return entries
  }
}
