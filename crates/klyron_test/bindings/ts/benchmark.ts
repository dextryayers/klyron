import { TestClient } from './client';

export function benchTestVersion(iterations: number): number {
  const client = new TestClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) { client.version(); }
  return Number(process.hrtime.bigint() - start) / 1e6;
}
