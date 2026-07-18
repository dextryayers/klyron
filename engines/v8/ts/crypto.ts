import type { NativeV8Bindings } from "./engine"

export class V8Crypto {
  private native: NativeV8Bindings
  private contextHandle: number

  constructor(native: NativeV8Bindings, contextHandle: number) {
    this.native = native
    this.contextHandle = contextHandle
  }

  randomBytes(size: number): Uint8Array {
    const handle = this.native.cryptoRandomBytes(this.contextHandle, size)
    const len = this.native.bufferGetLength(this.contextHandle, handle)
    const arr = new Uint8Array(len)
    for (let i = 0; i < len; i++) {
      arr[i] = this.native.getValueAtIndex(this.contextHandle, handle, i)
    }
    this.native.valueDispose(handle)
    return arr
  }

  randomUUID(): string {
    const result = this.native.cryptoRandomUUID(this.contextHandle)
    return result.data ?? ""
  }

  randomFillSync<T extends Uint8Array>(buffer: T): T {
    const bytes = this.randomBytes(buffer.length)
    buffer.set(bytes)
    return buffer
  }

  getRandomValues<T extends Uint8Array | Int8Array | number[]>(arr: T): T {
    const byteLen = (arr as Uint8Array).byteLength || (arr as number[]).length
    const bytes = this.randomBytes(byteLen)
    for (let i = 0; i < byteLen; i++) {
      (arr as number[])[i] = bytes[i]
    }
    return arr
  }

  createHash(_algorithm: string): V8Hash {
    return new V8Hash()
  }

  timingSafeEqual(a: Uint8Array, b: Uint8Array): boolean {
    if (a.length !== b.length) return false
    let result = 0
    for (let i = 0; i < a.length; i++) {
      result |= a[i] ^ b[i]
    }
    return result === 0
  }
}

export class V8Hash {
  private data: number[] = []

  update(input: string | Uint8Array): this {
    if (typeof input === "string") {
      for (let i = 0; i < input.length; i++) this.data.push(input.charCodeAt(i) & 0xff)
    } else {
      for (let i = 0; i < input.length; i++) this.data.push(input[i])
    }
    return this
  }

  digest(_encoding?: string): string | Uint8Array {
    const hash = this.simpleHash()
    return _encoding === "hex" ? hash : new Uint8Array(hash.match(/../g)?.map(h => parseInt(h, 16)) ?? [])
  }

  private simpleHash(): string {
    let h = 0
    for (const b of this.data) {
      h = ((h << 5) - h) + b
      h |= 0
    }
    const hex = (h >>> 0).toString(16).padStart(8, "0")
    return hex
  }
}
