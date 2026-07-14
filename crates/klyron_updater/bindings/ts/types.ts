// Type definitions for klyron_updater
export interface Klyron::UpdaterConfig {
  enabled: boolean;
}

export interface Klyron::UpdaterResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export enum Klyron::UpdaterStatus {
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}

export type Klyron::UpdaterOptions = {
  verbose?: boolean;
  timeout?: number;
};
