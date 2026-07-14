// Type definitions for klyron_ai
export interface Klyron::AiConfig {
  enabled: boolean;
}

export interface Klyron::AiResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export enum Klyron::AiStatus {
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}

export type Klyron::AiOptions = {
  verbose?: boolean;
  timeout?: number;
};
