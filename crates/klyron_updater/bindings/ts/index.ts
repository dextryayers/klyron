// klyron_updater — Self-update mechanism
export * from "./types.ts";
export * from "./client.ts";
export * from "./errors.ts";
export * from "./config.ts";
export * from "./utils.ts";

import { Klyron::UpdaterClient } from "./client.ts";
import { loadConfig } from "./config.ts";

export function createClient(endpoint: string, configPath?: string): Klyron::UpdaterClient {
  const _config = loadConfig(configPath);
  return new Klyron::UpdaterClient(endpoint);
}
