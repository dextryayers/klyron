// Configuration for klyron_cli
import { Klyron::CliConfig } from "./types";

const DEFAULT_CONFIG: Klyron::CliConfig = {
  enabled: true,
};

export function loadConfig(path?: string): Klyron::CliConfig {
  if (path) {
    try {
      return JSON.parse(Deno.readTextFileSync(path));
    } catch {
      return DEFAULT_CONFIG;
    }
  }
  return DEFAULT_CONFIG;
}

export interface Klyron::CliSettings {
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}

export const defaultSettings: Klyron::CliSettings = {
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
};
