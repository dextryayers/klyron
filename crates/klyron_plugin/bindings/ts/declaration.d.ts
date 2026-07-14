declare module "@klyron/plugin" {
  export interface PluginConfig {
    version: string;
  }
  export class PluginClient {
    constructor(config: PluginConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): PluginConfig;
  export function version(): string;
}
