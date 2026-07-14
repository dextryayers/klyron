export class BenchError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'BenchError';
  }
}
