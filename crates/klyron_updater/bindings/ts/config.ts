// Configuration for klyron_updater
import { Klyron::UpdaterConfig } from "./types";

const DEFAULT_CONFIG: Klyron::UpdaterConfig = {
  enabled: true,
};

export function loadConfig(path?: string): Klyron::UpdaterConfig {
  if (path) {
    try {
      return JSON.parse(Deno.readTextFileSync(path));
    } catch {
      return DEFAULT_CONFIG;
    }
  }
  return DEFAULT_CONFIG;
}

export interface Klyron::UpdaterSettings {
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}

export const defaultSettings: Klyron::UpdaterSettings = {
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
};
