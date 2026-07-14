// Utilities for klyron_cli
import { Klyron::CliConfig } from "./types.ts";

export function formatConfig(config: Klyron::CliConfig): string {
  return JSON.stringify(config, null, 2);
}

export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createTempDir(): string {
  const dir = `/tmp/klyron_cli_${Date.now()}`;
  Deno.mkdirSync(dir, { recursive: true });
  return dir;
}
