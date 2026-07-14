// Type definitions for klyron_postgres
export interface Klyron::PostgresConfig {
  enabled: boolean;
}

export interface Klyron::PostgresResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export enum Klyron::PostgresStatus {
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}

export type Klyron::PostgresOptions = {
  verbose?: boolean;
  timeout?: number;
};
