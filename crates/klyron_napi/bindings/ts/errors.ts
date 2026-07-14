export class NapiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'NapiError';
  }
}

export class ModuleNotFoundError extends NapiError {
  constructor(moduleName: string) {
    super(`N-API module '${moduleName}' not found`);
    this.name = 'ModuleNotFoundError';
  }
}

export class LoadFailedError extends NapiError {
  constructor(moduleName: string, reason: string) {
    super(`Failed to load '${moduleName}': ${reason}`);
    this.name = 'LoadFailedError';
  }
}

export class VersionMismatchError extends NapiError {
  constructor(expected: number, got: number) {
    super(`N-API version mismatch: expected ${expected}, got ${got}`);
    this.name = 'VersionMismatchError';
  }
}
