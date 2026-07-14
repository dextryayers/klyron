import { WatcherConfig } from './types';

export class WatcherClient {
  private config: WatcherConfig;

  constructor(config?: Partial<WatcherConfig>) {
    this.config = { enabled: true, verbose: false, ...config };
  }

  version(): string { return '1.0.0'; }

  getConfig(): WatcherConfig { return this.config; }
}
