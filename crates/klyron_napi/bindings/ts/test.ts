import { NapiClient } from './client';
import { NapiModule } from './types';

export function createTestClient(): NapiClient {
  return new NapiClient();
}

export function createMockModule(name: string): NapiModule {
  return { name, exports: {} };
}

export function assertModuleLoaded(client: NapiClient, name: string): void {
  if (!client.isLoaded(name)) {
    throw new Error(`Expected module '${name}' to be loaded`);
  }
}

export function assertModuleNotLoaded(client: NapiClient, name: string): void {
  if (client.isLoaded(name)) {
    throw new Error(`Expected module '${name}' to not be loaded`);
  }
}
