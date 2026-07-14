import { FileSystemClient } from './client';

export function createTestFileSystem(): FileSystemClient {
  return new FileSystemClient();
}

export function assertOk(value: boolean): void {
  if (!value) throw new Error('Assertion failed');
}
