// Type definitions for klyron_sqlite
export interface Klyron::SqliteConfig {
  enabled: boolean;
}

export interface Klyron::SqliteResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export enum Klyron::SqliteStatus {
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}

export type Klyron::SqliteOptions = {
  verbose?: boolean;
  timeout?: number;
};
