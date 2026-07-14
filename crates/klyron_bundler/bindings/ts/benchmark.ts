import { BundlerClient } from './client';

export function benchBundlerVersion(iterations: number): number {
  const client = new BundlerClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) { client.version(); }
  return Number(process.hrtime.bigint() - start) / 1e6;
}
