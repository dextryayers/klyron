export class BundlerError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'BundlerError';
  }
}
