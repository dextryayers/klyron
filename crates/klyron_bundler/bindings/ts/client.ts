import { BundlerConfig } from './types';

export class BundlerClient {
  private config: BundlerConfig;

  constructor(config?: Partial<BundlerConfig>) {
    this.config = { enabled: true, verbose: false, ...config };
  }

  version(): string { return '1.0.0'; }

  getConfig(): BundlerConfig { return this.config; }
}
