declare module "@klyron/adapter" {
  export interface AdapterConfig {
    version: string;
  }
  export class AdapterClient {
    constructor(config: AdapterConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): AdapterConfig;
  export function version(): string;
}
