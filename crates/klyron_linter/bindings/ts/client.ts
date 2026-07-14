import { LinterConfig } from './types';

export class LinterClient {
  private config: LinterConfig;

  constructor(config?: Partial<LinterConfig>) {
    this.config = { enabled: true, verbose: false, ...config };
  }

  version(): string { return '1.0.0'; }

  getConfig(): LinterConfig { return this.config; }
}
