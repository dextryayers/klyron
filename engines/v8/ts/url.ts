export interface URLParts {
  href: string
  protocol: string
  hostname: string
  port: string
  pathname: string
  search: string
  hash: string
  host: string
  origin: string
}

export class V8URL {
  private _href: string
  private _protocol = ""
  private _hostname = ""
  private _port = ""
  private _pathname = "/"
  private _search = ""
  private _hash = ""
  private _host = ""
  private _origin = "null"

  constructor(url: string, base?: string) {
    this._href = url
    try {
      this.parse(url, base)
    } catch {
      throw new TypeError(`Invalid URL: ${url}`)
    }
  }

  private parse(url: string, base?: string) {
    let target = url
    if (!target && base) target = base
    if (!target) throw new Error("Empty URL")

    const protoMatch = target.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):\/\//)
    if (protoMatch) {
      this._protocol = protoMatch[1]
      const rest = target.slice(protoMatch[0].length)
      this.parseHost(rest)
    } else if (target.startsWith("/")) {
      this._pathname = target
    } else if (base) {
      const baseURL = new V8URL(base)
      this._protocol = baseURL._protocol
      this._hostname = baseURL._hostname
      this._port = baseURL._port
      this._host = baseURL._host
      this._origin = baseURL._origin
      if (target.startsWith("?")) {
        this._search = target
      } else if (target.startsWith("#")) {
        this._hash = target
      } else {
        this._pathname = this.resolvePath(baseURL._pathname, target)
      }
    }

    this._href = this.toString()
    this._origin = this._protocol ? `${this._protocol}://${this._host}` : "null"
  }

  private parseHost(rest: string) {
    const slashIdx = rest.indexOf("/")
    const qIdx = rest.indexOf("?")
    const hIdx = rest.indexOf("#")
    let hostEnd = rest.length
    let pathStart = rest.length
    if (slashIdx >= 0) { hostEnd = Math.min(hostEnd, slashIdx); pathStart = slashIdx }
    if (qIdx >= 0) hostEnd = Math.min(hostEnd, qIdx)
    if (hIdx >= 0) hostEnd = Math.min(hostEnd, hIdx)

    this._host = rest.slice(0, hostEnd)
    const colonIdx = this._host.lastIndexOf(":")
    if (colonIdx >= 0) {
      this._hostname = this._host.slice(0, colonIdx)
      this._port = this._host.slice(colonIdx + 1)
    } else {
      this._hostname = this._host
    }

    let remaining = rest.slice(pathStart)
    const qIdx2 = remaining.indexOf("?")
    const hIdx2 = remaining.indexOf("#")
    let pathEnd = remaining.length
    if (qIdx2 >= 0) pathEnd = Math.min(pathEnd, qIdx2)
    if (hIdx2 >= 0) pathEnd = Math.min(pathEnd, hIdx2)
    this._pathname = remaining.slice(0, pathEnd) || "/"
    if (qIdx2 >= 0) {
      const searchEnd = hIdx2 >= 0 ? hIdx2 : remaining.length
      this._search = remaining.slice(qIdx2, searchEnd)
    }
    if (hIdx2 >= 0) {
      this._hash = remaining.slice(hIdx2)
    }
  }

  private resolvePath(base: string, relative: string): string {
    if (relative.startsWith("/")) return relative
    const baseParts = base.split("/").filter(Boolean)
    baseParts.pop()
    const relParts = relative.split("/").filter(Boolean)
    for (const part of relParts) {
      if (part === "..") baseParts.pop()
      else if (part !== ".") baseParts.push(part)
    }
    return "/" + baseParts.join("/")
  }

  get href(): string { return this._href }
  get protocol(): string { return this._protocol ? `${this._protocol}:` : "" }
  get hostname(): string { return this._hostname }
  get port(): string { return this._port }
  get pathname(): string { return this._pathname }
  get search(): string { return this._search }
  get hash(): string { return this._hash }
  get host(): string { return this._host }
  get origin(): string { return this._origin }

  toString(): string {
    let result = this._protocol ? `${this._protocol}://` : ""
    result += this._host
    if (this._pathname) result += this._pathname
    if (this._search) result += this._search
    if (this._hash) result += this._hash
    return result || "/"
  }

  toParts(): URLParts {
    return {
      href: this._href,
      protocol: this.protocol,
      hostname: this._hostname,
      port: this._port,
      pathname: this._pathname,
      search: this._search,
      hash: this._hash,
      host: this._host,
      origin: this._origin,
    }
  }

  static parse(url: string, base?: string): URLParts | null {
    try {
      return new V8URL(url, base).toParts()
    } catch {
      return null
    }
  }
}
