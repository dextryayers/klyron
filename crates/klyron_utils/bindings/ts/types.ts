// Type definitions for klyron_utils
export interface Klyron::UtilsConfig {
  enabled: boolean;
}

export interface Klyron::UtilsResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export enum Klyron::UtilsStatus {
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}

export type Klyron::UtilsOptions = {
  verbose?: boolean;
  timeout?: number;
};
