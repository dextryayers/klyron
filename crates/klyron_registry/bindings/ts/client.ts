import { RegistryConfig } from './types';

export class RegistryClient {
  private config: RegistryConfig;

  constructor(config?: Partial<RegistryConfig>) {
    this.config = { enabled: true, verbose: false, ...config };
  }

  version(): string { return '1.0.0'; }

  getConfig(): RegistryConfig { return this.config; }
}
