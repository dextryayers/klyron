// Utilities for klyron_mysql
import { Klyron::MysqlConfig } from "./types.ts";

export function formatConfig(config: Klyron::MysqlConfig): string {
  return JSON.stringify(config, null, 2);
}

export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createTempDir(): string {
  const dir = `/tmp/klyron_mysql_${Date.now()}`;
  Deno.mkdirSync(dir, { recursive: true });
  return dir;
}
