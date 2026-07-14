import { NapiClient } from './client';

export function benchLoad(iterations: number): number {
  const client = new NapiClient();
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) {
    client.version();
  }
  const end = process.hrtime.bigint();
  return Number(end - start) / 1e6;
}

export function benchIsNapiModule(iterations: number): number {
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) {
    NapiClient.isNapiModule('addon.node');
  }
  const end = process.hrtime.bigint();
  return Number(end - start) / 1e6;
}

export function runBenchmarks(): void {
  console.log(`load x1000: ${benchLoad(1000).toFixed(2)}ms`);
  console.log(`isNapiModule x10000: ${benchIsNapiModule(10000).toFixed(2)}ms`);
}
