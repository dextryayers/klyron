import type { NativeJSCBindings } from "./engine.ts";

export interface JSCStats {
  dev: number;
  ino: number;
  mode: number;
  nlink: number;
  uid: number;
  gid: number;
  rdev: number;
  size: number;
  blksize: number;
  blocks: number;
  atimeMs: number;
  mtimeMs: number;
  ctimeMs: number;
  isFile: boolean;
  isDirectory: boolean;
  isSymbolicLink: boolean;
}

export class JSCFS {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  readFile(path: string): number {
    return this.bindings.jscFSReadFile(this.handle, path);
  }

  writeFile(path: string, dataHandle: number): void {
    const r = this.bindings.jscFSWriteFile(this.handle, path, dataHandle);
    if (!r.success) throw new Error("writeFile failed");
  }

  stat(path: string): JSCStats {
    return this.bindings.jscFSStat(this.handle, path);
  }

  lstat(path: string): JSCStats {
    return this.bindings.jscFSLstat(this.handle, path);
  }

  mkdir(path: string, mode?: number): void {
    const r = this.bindings.jscFSMkdir(this.handle, path, mode ?? 0o755);
    if (!r.success) throw new Error("mkdir failed");
  }

  mkdirp(path: string, mode?: number): void {
    const r = this.bindings.jscFSMkdirp(this.handle, path, mode ?? 0o755);
    if (!r.success) throw new Error("mkdirp failed");
  }

  readdir(path: string): string[] {
    const r = this.bindings.jscFSReaddir(this.handle, path);
    return r;
  }

  unlink(path: string): void {
    const r = this.bindings.jscFSUnlink(this.handle, path);
    if (!r.success) throw new Error("unlink failed");
  }

  rmdir(path: string): void {
    const r = this.bindings.jscFSRmdir(this.handle, path);
    if (!r.success) throw new Error("rmdir failed");
  }

  rename(oldPath: string, newPath: string): void {
    const r = this.bindings.jscFSRename(this.handle, oldPath, newPath);
    if (!r.success) throw new Error("rename failed");
  }

  chmod(path: string, mode: number): void {
    const r = this.bindings.jscFSChmod(this.handle, path, mode);
    if (!r.success) throw new Error("chmod failed");
  }

  realpath(path: string): string {
    const r = this.bindings.jscFSRealpath(this.handle, path);
    return r;
  }

  exists(path: string): boolean {
    return this.bindings.jscFSExists(this.handle, path);
  }
}
