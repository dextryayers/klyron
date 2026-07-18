import type { NativeV8Bindings } from "./engine"

export class V8Buffer {
  private native: NativeV8Bindings
  private contextHandle: number
  private handle: number

  constructor(native: NativeV8Bindings, contextHandle: number, sizeOrData?: number | string | Uint8Array, encoding?: string) {
    this.native = native
    this.contextHandle = contextHandle

    if (typeof sizeOrData === "number") {
      this.handle = native.valueNewArrayBuffer(contextHandle, sizeOrData)
    } else if (typeof sizeOrData === "string") {
      const enc = encoding ?? "utf8"
      if (enc === "hex") {
        this.handle = native.encodingHexDecode(contextHandle, sizeOrData)
      } else if (enc === "base64") {
        this.handle = native.encodingBase64Decode(contextHandle, sizeOrData)
      } else {
        const encoded = native.encodingEncode(contextHandle, sizeOrData)
        this.handle = encoded
      }
    } else if (sizeOrData instanceof Uint8Array) {
      this.handle = native.bufferFromBytes(contextHandle, sizeOrData, sizeOrData.length)
    } else {
      this.handle = native.valueNewArrayBuffer(contextHandle, 0)
    }
  }

  get length(): number {
    return this.native.bufferGetLength(this.contextHandle, this.handle)
  }

  toString(encoding?: string, start?: number, end?: number): string {
    const enc = encoding ?? "utf8"
    const s = start ?? 0
    const e = end ?? this.length
    const result = this.native.bufferToString(this.contextHandle, this.handle, enc, s, e)
    return result.data ?? ""
  }

  toJSON(): { type: string; data: number[] } {
    return { type: "Buffer", data: Array.from(this.toBytes()) }
  }

  toBytes(): Uint8Array {
    const ptr = this.native.bufferGetData(this.contextHandle, this.handle)
    const len = this.length
    const arr = new Uint8Array(len)
    for (let i = 0; i < len; i++) arr[i] = this.native.getValueAtIndex(this.contextHandle, this.handle, i)
    return arr
  }

  slice(start: number, end?: number): V8Buffer {
    const e = end ?? this.length
    const handle = this.native.bufferSlice(this.contextHandle, this.handle, start, e)
    const buf = Object.create(V8Buffer.prototype) as V8Buffer
    buf.native = this.native
    buf.contextHandle = this.contextHandle
    buf.handle = handle
    return buf
  }

  copy(target: V8Buffer, targetStart?: number, sourceStart?: number, sourceEnd?: number): number {
    const ts = targetStart ?? 0
    const ss = sourceStart ?? 0
    const se = sourceEnd ?? this.length
    const count = se - ss
    const result = this.native.bufferCopy(this.contextHandle, target.handle, ts, this.handle, ss, count)
    return result.success ? count : 0
  }

  static concat(list: V8Buffer[], totalLength?: number): V8Buffer {
    if (list.length === 0) throw new Error("empty list")
    const first = list[0]
    const handles = list.map(b => b.handle)
    const handle = first.native.bufferConcat(first.contextHandle, handles, handles.length)
    const buf = Object.create(V8Buffer.prototype) as V8Buffer
    buf.native = first.native
    buf.contextHandle = first.contextHandle
    buf.handle = handle
    return buf
  }

  static from(data: string | Uint8Array | number[], encoding?: string): V8Buffer {
    if (typeof data === "string") return new V8Buffer(undefined as unknown as NativeV8Bindings, 0, data, encoding)
    if (data instanceof Uint8Array) return new V8Buffer(undefined as unknown as NativeV8Bindings, 0, data)
    return new V8Buffer(undefined as unknown as NativeV8Bindings, 0, new Uint8Array(data))
  }

  static alloc(size: number, fill?: number): V8Buffer {
    const buf = new V8Buffer(undefined as unknown as NativeV8Bindings, 0, size)
    if (fill !== undefined) buf.fill(fill)
    return buf
  }

  fill(value: number, offset?: number, end?: number): this {
    const o = offset ?? 0
    const e = end ?? this.length
    for (let i = o; i < e; i++) {
      this.native.setValueAtIndex(this.contextHandle, this.handle, i, value)
    }
    return this
  }

  write(data: string, offset?: number): number {
    const o = offset ?? 0
    const encoded = this.native.encodingEncode(this.contextHandle, data)
    const len = this.native.bufferGetLength(this.contextHandle, encoded)
    this.native.bufferCopy(this.contextHandle, this.handle, o, encoded, 0, len)
    return len
  }

  dispose(): void {
    this.native.valueDispose(this.handle)
  }
}
