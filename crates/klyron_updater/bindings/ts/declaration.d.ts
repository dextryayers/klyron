// Type declarations for klyron_updater
declare module "klyron/klyron_updater" {
  export interface Klyron::UpdaterConfig {
    enabled: boolean;
  }

  export interface Klyron::UpdaterResult<T> {
    success: boolean;
    data: T | null;
    error: string | null;
  }

  export class Klyron::UpdaterClient {
    constructor(endpoint: string);
    connect(): Promise<Klyron::UpdaterResult<null>>;
    ping(): Promise<boolean>;
  }

  export function createClient(endpoint: string, configPath?: string): Klyron::UpdaterClient;
  export function loadConfig(path?: string): Klyron::UpdaterConfig;
}
