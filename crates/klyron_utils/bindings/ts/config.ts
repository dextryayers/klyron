// Configuration for klyron_utils
import { Klyron::UtilsConfig } from "./types";

const DEFAULT_CONFIG: Klyron::UtilsConfig = {
  enabled: true,
};

export function loadConfig(path?: string): Klyron::UtilsConfig {
  if (path) {
    try {
      return JSON.parse(Deno.readTextFileSync(path));
    } catch {
      return DEFAULT_CONFIG;
    }
  }
  return DEFAULT_CONFIG;
}

export interface Klyron::UtilsSettings {
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}

export const defaultSettings: Klyron::UtilsSettings = {
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
};
