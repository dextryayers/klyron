export class FormatterError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'FormatterError';
  }
}
