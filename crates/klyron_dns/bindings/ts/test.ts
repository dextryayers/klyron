import { DnsResolverClient } from './client';

export function createTestDnsResolver(): DnsResolverClient {
  return new DnsResolverClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
