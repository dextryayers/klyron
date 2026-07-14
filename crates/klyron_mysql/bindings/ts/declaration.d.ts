// Type declarations for klyron_mysql
declare module "klyron/klyron_mysql" {
  export interface Klyron::MysqlConfig {
    enabled: boolean;
  }

  export interface Klyron::MysqlResult<T> {
    success: boolean;
    data: T | null;
    error: string | null;
  }

  export class Klyron::MysqlClient {
    constructor(endpoint: string);
    connect(): Promise<Klyron::MysqlResult<null>>;
    ping(): Promise<boolean>;
  }

  export function createClient(endpoint: string, configPath?: string): Klyron::MysqlClient;
  export function loadConfig(path?: string): Klyron::MysqlConfig;
}
