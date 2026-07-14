import { HttpServerClient } from './client';

export function createTestHttpServer(): HttpServerClient {
  return new HttpServerClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
