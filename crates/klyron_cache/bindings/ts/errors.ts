export class CacheManagerError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CacheManagerError';
  }
}

export class CacheManagerHttpError extends CacheManagerError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'CacheManagerHttpError';
  }
}
