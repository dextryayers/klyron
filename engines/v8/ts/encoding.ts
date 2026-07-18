import type { NativeV8Bindings } from "./engine"

export class V8TextEncoder {
  private native: NativeV8Bindings
  private contextHandle: number

  constructor(native: NativeV8Bindings, contextHandle: number) {
    this.native = native
    this.contextHandle = contextHandle
  }

  get encoding(): string {
    return "utf-8"
  }

  encode(input?: string): Uint8Array {
    if (!input || input.length === 0) return new Uint8Array(0)
    const handle = this.native.encodingEncode(this.contextHandle, input)
    const len = this.native.bufferGetLength(this.contextHandle, handle)
    const arr = new Uint8Array(len)
    for (let i = 0; i < len; i++) {
      arr[i] = this.native.getValueAtIndex(this.contextHandle, handle, i)
    }
    this.native.valueDispose(handle)
    return arr
  }

  encodeInto(source: string, destination: Uint8Array): { read: number; written: number } {
    let written = 0
    for (let i = 0; i < source.length && written < destination.length; i++) {
      const code = source.charCodeAt(i)
      if (code < 0x80) {
        destination[written++] = code
      } else if (code < 0x800) {
        if (written + 1 >= destination.length) { written--; break }
        destination[written++] = 0xc0 | (code >> 6)
        destination[written++] = 0x80 | (code & 0x3f)
      } else {
        if (written + 2 >= destination.length) { written--; break }
        destination[written++] = 0xe0 | (code >> 12)
        destination[written++] = 0x80 | ((code >> 6) & 0x3f)
        destination[written++] = 0x80 | (code & 0x3f)
      }
    }
    return { read: written, written }
  }
}

export class V8TextDecoder {
  private native: NativeV8Bindings
  private contextHandle: number
  private _encoding: string

  constructor(native: NativeV8Bindings, contextHandle: number, label = "utf-8") {
    this.native = native
    this.contextHandle = contextHandle
    this._encoding = label
  }

  get encoding(): string {
    return this._encoding
  }

  decode(input?: Uint8Array | ArrayBuffer, options?: { stream?: boolean }): string {
    if (!input) return ""
    let data: Uint8Array
    if (input instanceof ArrayBuffer) {
      data = new Uint8Array(input)
    } else {
      data = input
    }
    if (this._encoding === "utf-8" || this._encoding === "utf8") {
      return new TextDecoder().decode(data)
    }
    const result = this.native.encodingDecode(this.contextHandle, data, data.length, this._encoding)
    return result.data ?? ""
  }
}

export function base64Encode(data: Uint8Array): string {
  let binary = ""
  for (let i = 0; i < data.length; i++) {
    binary += String.fromCharCode(data[i])
  }
  return btoa(binary)
}

export function base64Decode(encoded: string): Uint8Array {
  const binary = atob(encoded)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}

export function hexEncode(data: Uint8Array): string {
  return Array.from(data).map(b => b.toString(16).padStart(2, "0")).join("")
}

export function hexDecode(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2)
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substr(i * 2, 2), 16)
  }
  return bytes
}
