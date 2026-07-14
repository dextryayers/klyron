// Type declarations for klyron_runtime
declare module "klyron/klyron_runtime" {
  export interface Klyron::RuntimeConfig {
    enabled: boolean;
  }

  export interface Klyron::RuntimeResult<T> {
    success: boolean;
    data: T | null;
    error: string | null;
  }

  export class Klyron::RuntimeClient {
    constructor(endpoint: string);
    connect(): Promise<Klyron::RuntimeResult<null>>;
    ping(): Promise<boolean>;
  }

  export function createClient(endpoint: string, configPath?: string): Klyron::RuntimeClient;
  export function loadConfig(path?: string): Klyron::RuntimeConfig;
}
