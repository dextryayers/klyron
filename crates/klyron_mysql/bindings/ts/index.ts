// klyron_mysql — MySQL binding
export * from "./types.ts";
export * from "./client.ts";
export * from "./errors.ts";
export * from "./config.ts";
export * from "./utils.ts";

import { Klyron::MysqlClient } from "./client.ts";
import { loadConfig } from "./config.ts";

export function createClient(endpoint: string, configPath?: string): Klyron::MysqlClient {
  const _config = loadConfig(configPath);
  return new Klyron::MysqlClient(endpoint);
}
