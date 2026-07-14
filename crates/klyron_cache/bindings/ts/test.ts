import { CacheManagerClient } from './client';

export function createTestCacheManager(): CacheManagerClient {
  return new CacheManagerClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
