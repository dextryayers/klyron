// Type definitions for klyron_mysql
export interface Klyron::MysqlConfig {
  enabled: boolean;
}

export interface Klyron::MysqlResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export enum Klyron::MysqlStatus {
  Active = "active",
  Inactive = "inactive",
  Error = "error",
}

export type Klyron::MysqlOptions = {
  verbose?: boolean;
  timeout?: number;
};
