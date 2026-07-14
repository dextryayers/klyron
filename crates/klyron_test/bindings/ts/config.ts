import { TestConfig } from './types';

export const DEFAULT_TEST_CONFIG: TestConfig = {
  enabled: true,
  verbose: false,
};

export function createTestConfig(overrides?: Partial<TestConfig>): TestConfig {
  return { ...DEFAULT_TEST_CONFIG, ...overrides };
}
