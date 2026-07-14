// Type declarations for klyron_cli
declare module "klyron/klyron_cli" {
  export interface Klyron::CliConfig {
    enabled: boolean;
  }

  export interface Klyron::CliResult<T> {
    success: boolean;
    data: T | null;
    error: string | null;
  }

  export class Klyron::CliClient {
    constructor(endpoint: string);
    connect(): Promise<Klyron::CliResult<null>>;
    ping(): Promise<boolean>;
  }

  export function createClient(endpoint: string, configPath?: string): Klyron::CliClient;
  export function loadConfig(path?: string): Klyron::CliConfig;
}
