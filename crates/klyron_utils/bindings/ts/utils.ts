// Utilities for klyron_utils
import { Klyron::UtilsConfig } from "./types.ts";

export function formatConfig(config: Klyron::UtilsConfig): string {
  return JSON.stringify(config, null, 2);
}

export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createTempDir(): string {
  const dir = `/tmp/klyron_utils_${Date.now()}`;
  Deno.mkdirSync(dir, { recursive: true });
  return dir;
}
