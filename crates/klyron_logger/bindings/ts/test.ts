import { LoggerClient } from './client';

export function createTestLogger(): LoggerClient {
  return new LoggerClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
