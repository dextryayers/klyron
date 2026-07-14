// Type declarations for klyron_engine
declare module "klyron/klyron_engine" {
  export interface Klyron::EngineConfig {
    enabled: boolean;
  }

  export interface Klyron::EngineResult<T> {
    success: boolean;
    data: T | null;
    error: string | null;
  }

  export class Klyron::EngineClient {
    constructor(endpoint: string);
    connect(): Promise<Klyron::EngineResult<null>>;
    ping(): Promise<boolean>;
  }

  export function createClient(endpoint: string, configPath?: string): Klyron::EngineClient;
  export function loadConfig(path?: string): Klyron::EngineConfig;
}
