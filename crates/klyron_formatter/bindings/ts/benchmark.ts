import { FormatterClient } from './client';

export function benchFormatterVersion(iterations: number): number {
  const client = new FormatterClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) { client.version(); }
  return Number(process.hrtime.bigint() - start) / 1e6;
}
