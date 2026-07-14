declare module "@klyron/compat" {
  export interface CompatConfig {
    version: string;
  }
  export class CompatClient {
    constructor(config: CompatConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): CompatConfig;
  export function version(): string;
}
