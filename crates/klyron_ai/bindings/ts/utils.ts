// Utilities for klyron_ai
import { Klyron::AiConfig } from "./types.ts";

export function formatConfig(config: Klyron::AiConfig): string {
  return JSON.stringify(config, null, 2);
}

export function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function createTempDir(): string {
  const dir = `/tmp/klyron_ai_${Date.now()}`;
  Deno.mkdirSync(dir, { recursive: true });
  return dir;
}
