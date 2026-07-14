export class LinterError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'LinterError';
  }
}
