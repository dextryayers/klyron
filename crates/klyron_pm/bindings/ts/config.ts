import { PmConfig } from './types';

export const DEFAULT_PM_CONFIG: PmConfig = {
  enabled: true,
  verbose: false,
};

export function createPmConfig(overrides?: Partial<PmConfig>): PmConfig {
  return { ...DEFAULT_PM_CONFIG, ...overrides };
}
