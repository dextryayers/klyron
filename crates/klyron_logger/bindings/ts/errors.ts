export class LoggerError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'LoggerError';
  }
}

export class LoggerHttpError extends LoggerError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'LoggerHttpError';
  }
}
