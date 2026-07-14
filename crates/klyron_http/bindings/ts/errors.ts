export class HttpServerError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'HttpServerError';
  }
}

export class HttpServerHttpError extends HttpServerError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'HttpServerHttpError';
  }
}
