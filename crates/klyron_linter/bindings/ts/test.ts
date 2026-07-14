import { LinterClient } from './client';

export function createTestLinterClient(): LinterClient {
  return new LinterClient();
}
