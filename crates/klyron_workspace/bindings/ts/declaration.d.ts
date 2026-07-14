declare module "@klyron/workspace" {
  export interface WorkspaceConfig {
    version: string;
  }
  export class WorkspaceClient {
    constructor(config: WorkspaceConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): WorkspaceConfig;
  export function version(): string;
}
