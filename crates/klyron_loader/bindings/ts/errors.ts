export class ModuleResolverError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ModuleResolverError';
  }
}

export class ModuleResolverHttpError extends ModuleResolverError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'ModuleResolverHttpError';
  }
}
