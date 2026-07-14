import { RegistryClient } from './client';

export function benchRegistryVersion(iterations: number): number {
  const client = new RegistryClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) { client.version(); }
  return Number(process.hrtime.bigint() - start) / 1e6;
}
