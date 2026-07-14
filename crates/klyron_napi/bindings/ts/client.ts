import { NapiModule, NapiModuleInfo } from './types';

export class NapiClient {
  private modules: Map<string, NapiModule> = new Map();

  load(name: string): NapiModule {
    const existing = this.modules.get(name);
    if (existing) return existing;
    const mod: NapiModule = { name, exports: {} };
    this.modules.set(name, mod);
    return mod;
  }

  list(): string[] {
    return Array.from(this.modules.keys());
  }

  unload(name: string): boolean {
    return this.modules.delete(name);
  }

  clear(): void {
    this.modules.clear();
  }

  isLoaded(name: string): boolean {
    return this.modules.has(name);
  }

  info(): NapiModuleInfo[] {
    return Array.from(this.modules.entries()).map(([name, mod]) => ({
      name,
      loaded: true,
      symbolCount: Object.keys(mod.exports).length,
    }));
  }

  version(): number {
    return 9;
  }

  static isNapiModule(name: string): boolean {
    return name.endsWith('.node');
  }
}
