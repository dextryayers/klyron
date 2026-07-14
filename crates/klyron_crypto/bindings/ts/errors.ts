export class CryptoProviderError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CryptoProviderError';
  }
}

export class CryptoProviderHttpError extends CryptoProviderError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'CryptoProviderHttpError';
  }
}
