// Configuration for klyron_runtime
import { Klyron::RuntimeConfig } from "./types";

const DEFAULT_CONFIG: Klyron::RuntimeConfig = {
  enabled: true,
};

export function loadConfig(path?: string): Klyron::RuntimeConfig {
  if (path) {
    try {
      return JSON.parse(Deno.readTextFileSync(path));
    } catch {
      return DEFAULT_CONFIG;
    }
  }
  return DEFAULT_CONFIG;
}

export interface Klyron::RuntimeSettings {
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}

export const defaultSettings: Klyron::RuntimeSettings = {
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
};
