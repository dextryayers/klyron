declare module "@klyron/deploy" {
  export interface DeployConfig {
    version: string;
  }
  export class DeployClient {
    constructor(config: DeployConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): DeployConfig;
  export function version(): string;
}
