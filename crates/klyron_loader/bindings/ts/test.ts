import { ModuleResolverClient } from './client';

export function createTestModuleResolver(): ModuleResolverClient {
  return new ModuleResolverClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
