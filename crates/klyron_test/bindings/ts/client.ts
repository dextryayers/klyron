import { TestConfig } from './types';

export class TestClient {
  private config: TestConfig;

  constructor(config?: Partial<TestConfig>) {
    this.config = { enabled: true, verbose: false, ...config };
  }

  version(): string { return '1.0.0'; }

  getConfig(): TestConfig { return this.config; }
}
