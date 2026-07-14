// klyron_engine — Polyglot engine trait + bridges
export * from "./types.ts";
export * from "./client.ts";
export * from "./errors.ts";
export * from "./config.ts";
export * from "./utils.ts";

import { Klyron::EngineClient } from "./client.ts";
import { loadConfig } from "./config.ts";

export function createClient(endpoint: string, configPath?: string): Klyron::EngineClient {
  const _config = loadConfig(configPath);
  return new Klyron::EngineClient(endpoint);
}
