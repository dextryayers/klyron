import type { NativeV8Bindings } from "./engine"

export type StreamData = string | Uint8Array | Buffer | null

export interface ReadableOptions {
  highWaterMark?: number
  encoding?: string
  objectMode?: boolean
  read?: (size: number) => void
}

export interface WritableOptions {
  highWaterMark?: number
  decodeStrings?: boolean
  objectMode?: boolean
  write?: (chunk: StreamData, encoding: string, callback: () => void) => void
  final?: (callback: () => void) => void
}

export interface TransformOptions {
  highWaterMark?: number
  objectMode?: boolean
  transform?: (chunk: StreamData, encoding: string, callback: () => void) => void
  flush?: (callback: () => void) => void
}

export class V8Readable {
  private _readable = true
  private _writable = false
  private _ended = false
  private _errored: Error | null = null
  private _buffer: StreamData[] = []
  private _highWaterMark: number
  private _objectMode: boolean
  private _readCallback: ((size: number) => void) | null = null
  private _encoding: string | null = null

  constructor(opts?: ReadableOptions) {
    this._highWaterMark = opts?.highWaterMark ?? 16384
    this._objectMode = opts?.objectMode ?? false
    this._encoding = opts?.encoding ?? null
    this._readCallback = opts?.read ?? null
  }

  get readable(): boolean { return !this._ended && !this._errored }
  get ended(): boolean { return this._ended }
  get errored(): Error | null { return this._errored }

  push(chunk: StreamData): boolean {
    if (this._ended) return false
    if (chunk === null) {
      this._ended = true
      return false
    }
    this._buffer.push(chunk)
    return this._buffer.length <= this._highWaterMark
  }

  read(size?: number): StreamData {
    if (this._buffer.length === 0) {
      if (this._readCallback) this._readCallback(size ?? -1)
      return null
    }
    if (!this._objectMode && size && size > 0) {
      const chunks: Uint8Array[] = []
      let total = 0
      while (this._buffer.length > 0 && total < size) {
        const chunk = this._buffer[0]
        const bytes = this.toBytes(chunk)
        if (total + bytes.length > size) {
          const remaining = size - total
          chunks.push(bytes.slice(0, remaining))
          this._buffer[0] = bytes.slice(remaining)
          total = size
        } else {
          chunks.push(bytes)
          this._buffer.shift()
          total += bytes.length
        }
      }
      const combined = new Uint8Array(total)
      let offset = 0
      for (const c of chunks) { combined.set(c, offset); offset += c.length }
      return combined
    }
    return this._buffer.shift() ?? null
  }

  private toBytes(data: StreamData): Uint8Array {
    if (data === null || data === undefined) return new Uint8Array(0)
    if (data instanceof Uint8Array) return data
    if (typeof data === "string") return new TextEncoder().encode(data)
    return new Uint8Array(data as ArrayBuffer)
  }

  pipe(dest: V8Writable): V8Writable {
    let chunk: StreamData
    while ((chunk = this.read()) !== null) {
      dest.write(chunk)
    }
    this.on("end", () => dest.end())
    return dest
  }

  private listeners: Map<string, Array<(...args: unknown[]) => void>> = new Map()

  on(event: string, cb: (...args: unknown[]) => void): this {
    if (!this.listeners.has(event)) this.listeners.set(event, [])
    this.listeners.get(event)!.push(cb)
    return this
  }

  emit(event: string, ...args: unknown[]): boolean {
    const listeners = this.listeners.get(event)
    if (!listeners) return false
    for (const cb of listeners) cb(...args)
    return true
  }

  destroy(err?: Error): void {
    this._ended = true
    this._errored = err ?? null
    this._buffer = []
    this.emit("close")
  }
}

export class V8Writable {
  private _writable = true
  private _ended = false
  private _errored: Error | null = null
  private _writeCallback: ((chunk: StreamData, encoding: string, callback: () => void) => void) | null = null
  private _finalCallback: ((callback: () => void) => void) | null = null
  private _decodeStrings = true

  constructor(opts?: WritableOptions) {
    this._writeCallback = opts?.write ?? null
    this._finalCallback = opts?.final ?? null
    this._decodeStrings = opts?.decodeStrings ?? true
  }

  get writable(): boolean { return !this._ended && !this._errored }
  get ended(): boolean { return this._ended }

  write(chunk: StreamData, encoding?: string, cb?: () => void): boolean {
    if (this._ended) { cb?.(); return false }
    const enc = encoding ?? "utf8"
    if (this._writeCallback) {
      this._writeCallback(chunk, enc, cb ?? (() => {}))
    } else {
      cb?.()
    }
    return true
  }

  end(chunk?: StreamData, encoding?: string, cb?: () => void): void {
    if (this._ended) return
    if (chunk) this.write(chunk, encoding)
    this._ended = true
    if (this._finalCallback) {
      this._finalCallback(cb ?? (() => {}))
    } else {
      cb?.()
    }
    this.emit("finish")
  }

  private listeners: Map<string, Array<(...args: unknown[]) => void>> = new Map()

  on(event: string, cb: (...args: unknown[]) => void): this {
    if (!this.listeners.has(event)) this.listeners.set(event, [])
    this.listeners.get(event)!.push(cb)
    return this
  }

  emit(event: string, ...args: unknown[]): boolean {
    const listeners = this.listeners.get(event)
    if (!listeners) return false
    for (const cb of listeners) cb(...args)
    return true
  }

  destroy(err?: Error): void {
    this._ended = true
    this._errored = err ?? null
    this.emit("close")
  }
}

export class V8Transform {
  private _readable: V8Readable
  private _writable: V8Writable
  private _transformCallback: ((chunk: StreamData, encoding: string, callback: () => void) => void) | null = null
  private _flushCallback: ((callback: () => void) => void) | null = null
  private _ended = false

  constructor(opts?: TransformOptions) {
    this._readable = new V8Readable({ highWaterMark: opts?.highWaterMark, objectMode: opts?.objectMode })
    this._writable = new V8Writable({
      write: (chunk, encoding, callback) => {
        if (this._transformCallback) {
          this._transformCallback(chunk, encoding, callback)
        } else {
          this._readable.push(chunk)
          callback()
        }
      },
      final: (callback) => {
        this._ended = true
        if (this._flushCallback) {
          this._flushCallback(callback)
        } else {
          this._readable.push(null)
          callback()
        }
      },
    })
    this._transformCallback = opts?.transform ?? null
    this._flushCallback = opts?.flush ?? null
  }

  get readable(): V8Readable { return this._readable }
  get writable(): V8Writable { return this._writable }

  push(chunk: StreamData): boolean {
    return this._readable.push(chunk)
  }

  write(chunk: StreamData, encoding?: string, cb?: () => void): boolean {
    return this._writable.write(chunk, encoding, cb)
  }

  end(chunk?: StreamData, encoding?: string, cb?: () => void): void {
    this._writable.end(chunk, encoding, cb)
  }

  destroy(err?: Error): void {
    this._readable.destroy(err)
    this._writable.destroy(err)
  }
}
