// Type declarations for klyron_sqlite
declare module "klyron/klyron_sqlite" {
  export interface Klyron::SqliteConfig {
    enabled: boolean;
  }

  export interface Klyron::SqliteResult<T> {
    success: boolean;
    data: T | null;
    error: string | null;
  }

  export class Klyron::SqliteClient {
    constructor(endpoint: string);
    connect(): Promise<Klyron::SqliteResult<null>>;
    ping(): Promise<boolean>;
  }

  export function createClient(endpoint: string, configPath?: string): Klyron::SqliteClient;
  export function loadConfig(path?: string): Klyron::SqliteConfig;
}
