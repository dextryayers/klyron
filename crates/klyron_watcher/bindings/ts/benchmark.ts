import { WatcherClient } from './client';

export function benchWatcherVersion(iterations: number): number {
  const client = new WatcherClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) { client.version(); }
  return Number(process.hrtime.bigint() - start) / 1e6;
}
