import type { NativeJSCBindings } from "./engine.ts";

export interface JSCOSInfo {
  hostname: string;
  platform: string;
  arch: string;
  release: string;
  type: string;
  uptime: number;
  totalMemory: number;
  freeMemory: number;
  cpus: number;
  loadAvg: number[];
  homedir: string;
  tmpdir: string;
  user: { uid: number; gid: number; username: string; homedir: string; shell: string };
}

export class JSCOperatingSystem {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  hostname(): string {
    return this.bindings.jscOSHostname(this.handle);
  }

  platform(): string {
    return this.bindings.jscOSPlatform(this.handle);
  }

  arch(): string {
    return this.bindings.jscOSArch(this.handle);
  }

  release(): string {
    return this.bindings.jscOSRelease(this.handle);
  }

  type(): string {
    return this.bindings.jscOSType(this.handle);
  }

  uptime(): number {
    return this.bindings.jscOSUptime(this.handle);
  }

  totalMemory(): number {
    return this.bindings.jscOSTotalMemory(this.handle);
  }

  freeMemory(): number {
    return this.bindings.jscOSFreeMemory(this.handle);
  }

  cpus(): number {
    return this.bindings.jscOSCpus(this.handle);
  }

  loadAvg(): number[] {
    return this.bindings.jscOSLoadAvg(this.handle);
  }

  homedir(): string {
    return this.bindings.jscOSHomedir(this.handle);
  }

  tmpdir(): string {
    return this.bindings.jscOSTmpdir(this.handle);
  }

  userInfo(): { uid: number; gid: number; username: string; homedir: string; shell: string } {
    return this.bindings.jscOSUserInfo(this.handle);
  }

  info(): JSCOSInfo {
    return this.bindings.jscOSInfo(this.handle);
  }
}
