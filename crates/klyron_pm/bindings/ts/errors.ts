export class PmError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'PmError';
  }
}
