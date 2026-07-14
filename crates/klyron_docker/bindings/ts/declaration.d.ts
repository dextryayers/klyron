declare module "@klyron/docker" {
  export interface DockerConfig {
    version: string;
  }
  export class DockerClient {
    constructor(config: DockerConfig);
    execute(): Promise<void>;
  }
  export function loadConfig(): DockerConfig;
  export function version(): string;
}
