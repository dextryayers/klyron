declare module "@klyron/shell" {
  export interface ShellConfig {
    version: string;
  }
  export class ShellClient {
    constructor(config: ShellConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): ShellConfig;
  export function version(): string;
}
