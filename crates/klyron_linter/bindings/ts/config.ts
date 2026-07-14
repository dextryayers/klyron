import { LinterConfig } from './types';

export const DEFAULT_LINTER_CONFIG: LinterConfig = {
  enabled: true,
  verbose: false,
};

export function createLinterConfig(overrides?: Partial<LinterConfig>): LinterConfig {
  return { ...DEFAULT_LINTER_CONFIG, ...overrides };
}
