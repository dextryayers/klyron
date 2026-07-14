import { WebApiClient } from './client';

export function createTestWebApi(): WebApiClient {
  return new WebApiClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
