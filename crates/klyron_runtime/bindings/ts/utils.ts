// Utilities for klyron_runtime
import { Klyron::RuntimeConfig } from "./types.ts";

export function formatConfig(config: Klyron::RuntimeConfig): string {
  return JSON.stringify(config, null, 2);
}

export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createTempDir(): string {
  const dir = `/tmp/klyron_runtime_${Date.now()}`;
  Deno.mkdirSync(dir, { recursive: true });
  return dir;
}
