// Configuration for klyron_sqlite
import { Klyron::SqliteConfig } from "./types";

const DEFAULT_CONFIG: Klyron::SqliteConfig = {
  enabled: true,
};

export function loadConfig(path?: string): Klyron::SqliteConfig {
  if (path) {
    try {
      return JSON.parse(Deno.readTextFileSync(path));
    } catch {
      return DEFAULT_CONFIG;
    }
  }
  return DEFAULT_CONFIG;
}

export interface Klyron::SqliteSettings {
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}

export const defaultSettings: Klyron::SqliteSettings = {
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
};
