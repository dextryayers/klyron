// Type declarations for klyron_postgres
declare module "klyron/klyron_postgres" {
  export interface Klyron::PostgresConfig {
    enabled: boolean;
  }

  export interface Klyron::PostgresResult<T> {
    success: boolean;
    data: T | null;
    error: string | null;
  }

  export class Klyron::PostgresClient {
    constructor(endpoint: string);
    connect(): Promise<Klyron::PostgresResult<null>>;
    ping(): Promise<boolean>;
  }

  export function createClient(endpoint: string, configPath?: string): Klyron::PostgresClient;
  export function loadConfig(path?: string): Klyron::PostgresConfig;
}
