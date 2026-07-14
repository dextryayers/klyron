declare module "@klyron/config" {
  export interface ConfigConfig {
    version: string;
  }
  export class ConfigClient {
    constructor(config: ConfigConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): ConfigConfig;
  export function version(): string;
}
