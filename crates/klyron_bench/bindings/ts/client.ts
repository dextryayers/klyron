import { BenchConfig } from './types';

export class BenchClient {
  private config: BenchConfig;

  constructor(config?: Partial<BenchConfig>) {
    this.config = { enabled: true, verbose: false, ...config };
  }

  version(): string { return '1.0.0'; }

  getConfig(): BenchConfig { return this.config; }
}
