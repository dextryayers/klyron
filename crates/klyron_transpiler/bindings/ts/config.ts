import { TranspilerConfig } from './types';

export const DEFAULT_TRANSPILER_CONFIG: TranspilerConfig = {
  enabled: true,
  verbose: false,
};

export function createTranspilerConfig(overrides?: Partial<TranspilerConfig>): TranspilerConfig {
  return { ...DEFAULT_TRANSPILER_CONFIG, ...overrides };
}
