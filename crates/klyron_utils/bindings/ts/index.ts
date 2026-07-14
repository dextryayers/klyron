// klyron_utils — Shared utilities
export * from "./types.ts";
export * from "./client.ts";
export * from "./errors.ts";
export * from "./config.ts";
export * from "./utils.ts";

import { Klyron::UtilsClient } from "./client.ts";
import { loadConfig } from "./config.ts";

export function createClient(endpoint: string, configPath?: string): Klyron::UtilsClient {
  const _config = loadConfig(configPath);
  return new Klyron::UtilsClient(endpoint);
}
