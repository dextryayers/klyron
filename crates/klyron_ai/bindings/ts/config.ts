// Configuration for klyron_ai
import { Klyron::AiConfig } from "./types";

const DEFAULT_CONFIG: Klyron::AiConfig = {
  enabled: true,
};

export function loadConfig(path?: string): Klyron::AiConfig {
  if (path) {
    try {
      return JSON.parse(Deno.readTextFileSync(path));
    } catch {
      return DEFAULT_CONFIG;
    }
  }
  return DEFAULT_CONFIG;
}

export interface Klyron::AiSettings {
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}

export const defaultSettings: Klyron::AiSettings = {
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
};
