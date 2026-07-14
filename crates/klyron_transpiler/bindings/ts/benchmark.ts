import { TranspilerClient } from './client';

export function benchTranspilerVersion(iterations: number): number {
  const client = new TranspilerClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) { client.version(); }
  return Number(process.hrtime.bigint() - start) / 1e6;
}
