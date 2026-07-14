import { CryptoProviderClient } from './client';

export function createTestCryptoProvider(): CryptoProviderClient {
  return new CryptoProviderClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
