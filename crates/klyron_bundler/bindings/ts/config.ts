import { BundlerConfig } from './types';

export const DEFAULT_BUNDLER_CONFIG: BundlerConfig = {
  enabled: true,
  verbose: false,
};

export function createBundlerConfig(overrides?: Partial<BundlerConfig>): BundlerConfig {
  return { ...DEFAULT_BUNDLER_CONFIG, ...overrides };
}
