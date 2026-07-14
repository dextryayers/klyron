import { PmConfig } from './types';

export class PmClient {
  private config: PmConfig;

  constructor(config?: Partial<PmConfig>) {
    this.config = { enabled: true, verbose: false, ...config };
  }

  version(): string { return '1.0.0'; }

  getConfig(): PmConfig { return this.config; }
}
