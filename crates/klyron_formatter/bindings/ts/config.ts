import { FormatterConfig } from './types';

export const DEFAULT_FORMATTER_CONFIG: FormatterConfig = {
  enabled: true,
  verbose: false,
};

export function createFormatterConfig(overrides?: Partial<FormatterConfig>): FormatterConfig {
  return { ...DEFAULT_FORMATTER_CONFIG, ...overrides };
}
