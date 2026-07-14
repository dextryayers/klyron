// Utilities for klyron_sqlite
import { Klyron::SqliteConfig } from "./types.ts";

export function formatConfig(config: Klyron::SqliteConfig): string {
  return JSON.stringify(config, null, 2);
}

export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createTempDir(): string {
  const dir = `/tmp/klyron_sqlite_${Date.now()}`;
  Deno.mkdirSync(dir, { recursive: true });
  return dir;
}
