import { BenchConfig } from './types';

export const DEFAULT_BENCH_CONFIG: BenchConfig = {
  enabled: true,
  verbose: false,
};

export function createBenchConfig(overrides?: Partial<BenchConfig>): BenchConfig {
  return { ...DEFAULT_BENCH_CONFIG, ...overrides };
}
