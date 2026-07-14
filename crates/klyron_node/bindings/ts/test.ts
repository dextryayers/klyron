import { NodeGlobalsClient } from './client';

export function createTestNodeGlobals(): NodeGlobalsClient {
  return new NodeGlobalsClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
