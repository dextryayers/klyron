import { RegistryConfig } from './types';

export const DEFAULT_REGISTRY_CONFIG: RegistryConfig = {
  enabled: true,
  verbose: false,
};

export function createRegistryConfig(overrides?: Partial<RegistryConfig>): RegistryConfig {
  return { ...DEFAULT_REGISTRY_CONFIG, ...overrides };
}
