// Type definitions for klyron_engine
export interface Klyron::EngineConfig {
  enabled: boolean;
}

export interface Klyron::EngineResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export enum Klyron::EngineStatus {
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}

export type Klyron::EngineOptions = {
  verbose?: boolean;
  timeout?: number;
};
