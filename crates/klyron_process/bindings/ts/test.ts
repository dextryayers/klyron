import { ProcessManagerClient } from './client';

export function createTestProcessManager(): ProcessManagerClient {
  return new ProcessManagerClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
