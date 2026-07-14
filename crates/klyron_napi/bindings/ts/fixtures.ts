import { NapiModule, NapiLoaderConfig } from './types';

export const TEST_MODULE_NAMES = ['test-addon', 'sample.node', 'native-bindings'];

export const MOCK_MODULES: Record<string, NapiModule> = {
  'test-addon': { name: 'test-addon', exports: { hello: 'world' } },
  'sample.node': { name: 'sample.node', exports: { add: (a: number, b: number) => a + b } },
};

export const TEST_CONFIGS: NapiLoaderConfig[] = [
  { modulePaths: ['node_modules'], cacheEnabled: true },
  { modulePaths: ['node_modules', './lib'], cacheEnabled: false },
];

export function getMockModule(name: string): NapiModule | undefined {
  return MOCK_MODULES[name];
}
