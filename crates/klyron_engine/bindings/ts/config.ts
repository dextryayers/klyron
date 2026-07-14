// Configuration for klyron_engine
import { Klyron::EngineConfig } from "./types";

const DEFAULT_CONFIG: Klyron::EngineConfig = {
  enabled: true,
};

export function loadConfig(path?: string): Klyron::EngineConfig {
  if (path) {
    try {
      return JSON.parse(Deno.readTextFileSync(path));
    } catch {
      return DEFAULT_CONFIG;
    }
  }
  return DEFAULT_CONFIG;
}

export interface Klyron::EngineSettings {
  maxRetries: number;
  timeoutMs: number;
  logLevel: string;
}

export const defaultSettings: Klyron::EngineSettings = {
  maxRetries: 3,
  timeoutMs: 5000,
  logLevel: "info",
};
