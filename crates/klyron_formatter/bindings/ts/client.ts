import { FormatterConfig } from './types';

export class FormatterClient {
  private config: FormatterConfig;

  constructor(config?: Partial<FormatterConfig>) {
    this.config = { enabled: true, verbose: false, ...config };
  }

  version(): string { return '1.0.0'; }

  getConfig(): FormatterConfig { return this.config; }
}
