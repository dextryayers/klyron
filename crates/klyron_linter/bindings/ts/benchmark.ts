import { LinterClient } from './client';

export function benchLinterVersion(iterations: number): number {
  const client = new LinterClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) { client.version(); }
  return Number(process.hrtime.bigint() - start) / 1e6;
}
