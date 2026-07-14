// Configuration for klyron_postgres
import { Klyron::PostgresConfig } from "./types";

const DEFAULT_CONFIG: Klyron::PostgresConfig = {
  enabled: true,
};

export function loadConfig(path?: string): Klyron::PostgresConfig {
  if (path) {
    try {
      return JSON.parse(Deno.readTextFileSync(path));
    } catch {
      return DEFAULT_CONFIG;
    }
  }
  return DEFAULT_CONFIG;
}

export interface Klyron::PostgresSettings {
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}

export const defaultSettings: Klyron::PostgresSettings = {
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
};
