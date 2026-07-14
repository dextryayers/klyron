import { BenchClient } from './client';

export function benchBenchVersion(iterations: number): number {
  const client = new BenchClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) { client.version(); }
  return Number(process.hrtime.bigint() - start) / 1e6;
}
