import type { NativeJSCBindings } from "./engine.ts";

export interface JSCProcessMemoryUsage {
  rss: number;
  heapTotal: number;
  heapUsed: number;
  external: number;
  arrayBuffers: number;
}

export class JSCProcess {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  get pid(): number {
    return this.bindings.jscProcessPid(this.handle);
  }

  get ppid(): number {
    return this.bindings.jscProcessPpid(this.handle);
  }

  get cwd(): string {
    const r = this.bindings.jscProcessCwdStr(this.handle);
    return r.data ?? "";
  }

  get execPath(): string {
    const r = this.bindings.jscProcessExecPath(this.handle);
    return r.data ?? "";
  }

  get platform(): string {
    return this.bindings.jscProcessPlatform(this.handle);
  }

  get arch(): string {
    return this.bindings.jscProcessArch(this.handle);
  }

  get title(): string {
    const r = this.bindings.jscProcessTitle(this.handle);
    return r.data ?? "";
  }

  get argv(): string[] {
    return this.bindings.jscProcessArgv(this.handle);
  }

  get env(): Record<string, string> {
    return this.bindings.jscProcessEnvAll(this.handle);
  }

  exit(code?: number): void {
    this.bindings.jscProcessExit(this.handle, code ?? 0);
  }

  memoryUsage(): JSCProcessMemoryUsage {
    return this.bindings.jscProcessMemoryUsage(this.handle);
  }

  uptime(): number {
    return this.bindings.jscProcessUptime(this.handle);
  }
}
