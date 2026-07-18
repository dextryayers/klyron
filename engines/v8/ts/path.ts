export class V8Path {
  static sep = "/"
  static delimiter = ":"

  static normalize(p: string): string {
    if (!p) return "."
    const isAbsolute = p.startsWith("/")
    const parts = p.split("/").filter(Boolean)
    const resolved: string[] = []
    for (const part of parts) {
      if (part === ".") continue
      if (part === "..") {
        if (resolved.length > 0 && resolved[resolved.length - 1] !== "..") {
          resolved.pop()
        } else if (!isAbsolute) {
          resolved.push("..")
        }
      } else {
        resolved.push(part)
      }
    }
    let result = resolved.join("/")
    if (isAbsolute) result = "/" + result
    if (p.endsWith("/") && result !== "/") result += "/"
    return result || "."
  }

  static join(...segments: string[]): string {
    const filtered = segments.filter(s => s.length > 0)
    if (filtered.length === 0) return "."
    const joined = filtered.join("/")
    return V8Path.normalize(joined)
  }

  static resolve(...segments: string[]): string {
    let resolved = ""
    let resolvedAbsolute = false
    for (let i = segments.length - 1; i >= 0 && !resolvedAbsolute; i--) {
      const s = segments[i]
      if (!s) continue
      resolved = s + "/" + resolved
      resolvedAbsolute = s.startsWith("/")
    }
    if (!resolvedAbsolute) {
      resolved = V8Path.normalize(resolved)
    }
    return V8Path.normalize(resolved)
  }

  static dirname(p: string): string {
    const normalized = V8Path.normalize(p)
    if (normalized === "/") return "/"
    const lastSlash = normalized.lastIndexOf("/")
    if (lastSlash === -1) return "."
    if (lastSlash === 0) return "/"
    return normalized.slice(0, lastSlash)
  }

  static basename(p: string, ext?: string): string {
    const normalized = V8Path.normalize(p)
    if (normalized === "/") return "/"
    const lastSlash = normalized.lastIndexOf("/")
    const base = lastSlash === -1 ? normalized : normalized.slice(lastSlash + 1)
    if (ext && base.endsWith(ext)) {
      return base.slice(0, -ext.length)
    }
    return base
  }

  static extname(p: string): string {
    const normalized = V8Path.normalize(p)
    const base = V8Path.basename(normalized)
    const dotIdx = base.lastIndexOf(".")
    if (dotIdx <= 0) return ""
    return base.slice(dotIdx)
  }

  static isAbsolute(p: string): boolean {
    return p.startsWith("/")
  }

  static relative(from: string, to: string): string {
    const fromParts = V8Path.normalize(from).split("/").filter(Boolean)
    const toParts = V8Path.normalize(to).split("/").filter(Boolean)
    let i = 0
    while (i < fromParts.length && i < toParts.length && fromParts[i] === toParts[i]) {
      i++
    }
    const up = fromParts.length - i
    const rel: string[] = []
    for (let j = 0; j < up; j++) rel.push("..")
    rel.push(...toParts.slice(i))
    return rel.join("/") || "."
  }

  static parse(p: string): {
    root: string
    dir: string
    base: string
    ext: string
    name: string
  } {
    const isAbsolute = p.startsWith("/")
    const dir = V8Path.dirname(p)
    const base = V8Path.basename(p)
    const ext = V8Path.extname(p)
    const name = ext ? base.slice(0, -ext.length) : base
    return {
      root: isAbsolute ? "/" : "",
      dir,
      base,
      ext,
      name,
    }
  }

  static format(parts: { root?: string; dir?: string; base?: string; ext?: string; name?: string }): string {
    let p = parts.dir ?? ""
    const base = parts.base ?? (parts.name ?? "") + (parts.ext ?? "")
    if (p && !p.endsWith("/")) p += "/"
    return p + base
  }
}
