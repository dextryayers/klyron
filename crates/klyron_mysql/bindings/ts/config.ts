// Configuration for klyron_mysql
import { Klyron::MysqlConfig } from "./types";

const DEFAULT_CONFIG: Klyron::MysqlConfig = {
  enabled: true,
};

export function loadConfig(path?: string): Klyron::MysqlConfig {
  if (path) {
    try {
      return JSON.parse(Deno.readTextFileSync(path));
    } catch {
      return DEFAULT_CONFIG;
    }
  }
  return DEFAULT_CONFIG;
}

export interface Klyron::MysqlSettings {
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}

export const defaultSettings: Klyron::MysqlSettings = {
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
};
