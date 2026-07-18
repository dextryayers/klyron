import type { NativeJSCBindings } from "./engine.ts";

export class JSCConsole {
  constructor(
    private handle: number,
    private bindings: NativeJSCBindings,
  ) {}

  log(...args: number[]): void {
    this.bindings.jscConsoleLog(this.handle, args);
  }

  warn(...args: number[]): void {
    this.bindings.jscConsoleWarn(this.handle, args);
  }

  error(...args: number[]): void {
    this.bindings.jscConsoleError(this.handle, args);
  }

  info(...args: number[]): void {
    this.bindings.jscConsoleInfo(this.handle, args);
  }

  debug(...args: number[]): void {
    this.bindings.jscConsoleDebug(this.handle, args);
  }

  table(data: number): void {
    this.bindings.jscConsoleTable(this.handle, data);
  }

  assert(condition: number, ...args: number[]): void {
    this.bindings.jscConsoleAssert(this.handle, condition, args);
  }

  count(label?: string): void {
    this.bindings.jscConsoleCount(this.handle, label ?? null);
  }

  time(label?: string): void {
    this.bindings.jscConsoleTime(this.handle, label ?? null);
  }

  timeEnd(label?: string): void {
    this.bindings.jscConsoleTimeEnd(this.handle, label ?? null);
  }

  trace(): void {
    this.bindings.jscConsoleTrace(this.handle);
  }

  group(label?: string): void {
    this.bindings.jscConsoleGroup(this.handle, label ?? null);
  }

  groupEnd(): void {
    this.bindings.jscConsoleGroupEnd(this.handle);
  }
}
