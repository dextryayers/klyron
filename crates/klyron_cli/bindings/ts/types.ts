// Type definitions for klyron_cli
export interface Klyron::CliConfig {
  enabled: boolean;
}

export interface Klyron::CliResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export enum Klyron::CliStatus {
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}

export type Klyron::CliOptions = {
  verbose?: boolean;
  timeout?: number;
};
