// Type declarations for klyron_ai
declare module "klyron/klyron_ai" {
  export interface Klyron::AiConfig {
    enabled: boolean;
  }

  export interface Klyron::AiResult<T> {
    success: boolean;
    data: T | null;
    error: string | null;
  }

  export class Klyron::AiClient {
    constructor(endpoint: string);
    connect(): Promise<Klyron::AiResult<null>>;
    ping(): Promise<boolean>;
  }

  export function createClient(endpoint: string, configPath?: string): Klyron::AiClient;
  export function loadConfig(path?: string): Klyron::AiConfig;
}
