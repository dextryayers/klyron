import { TranspilerConfig } from './types';

export class TranspilerClient {
  private config: TranspilerConfig;

  constructor(config?: Partial<TranspilerConfig>) {
    this.config = { enabled: true, verbose: false, ...config };
  }

  version(): string { return '1.0.0'; }

  getConfig(): TranspilerConfig { return this.config; }
}
