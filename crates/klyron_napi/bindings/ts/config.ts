import { NapiLoaderConfig } from './types';

export interface NapiConfig {
  loader: NapiLoaderConfig;
  napiVersion: number;
}

export const DEFAULT_NAPI_CONFIG: NapiConfig = {
  loader: {
    modulePaths: ['node_modules'],
    cacheEnabled: true,
  },
  napiVersion: 9,
};

export function createNapiConfig(
  overrides?: Partial<NapiConfig>
): NapiConfig {
  return { ...DEFAULT_NAPI_CONFIG, ...overrides };
}
