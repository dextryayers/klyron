// Type declarations for klyron_utils
declare module "klyron/klyron_utils" {
  export interface Klyron::UtilsConfig {
    enabled: boolean;
  }

  export interface Klyron::UtilsResult<T> {
    success: boolean;
    data: T | null;
    error: string | null;
  }

  export class Klyron::UtilsClient {
    constructor(endpoint: string);
    connect(): Promise<Klyron::UtilsResult<null>>;
    ping(): Promise<boolean>;
  }

  export function createClient(endpoint: string, configPath?: string): Klyron::UtilsClient;
  export function loadConfig(path?: string): Klyron::UtilsConfig;
}
