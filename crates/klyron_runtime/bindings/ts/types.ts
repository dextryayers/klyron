// Type definitions for klyron_runtime
export interface Klyron::RuntimeConfig {
  enabled: boolean;
}

export interface Klyron::RuntimeResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export enum Klyron::RuntimeStatus {
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}

export type Klyron::RuntimeOptions = {
  verbose?: boolean;
  timeout?: number;
};
