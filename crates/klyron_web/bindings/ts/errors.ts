export class WebApiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'WebApiError';
  }
}

export class WebApiHttpError extends WebApiError {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'WebApiHttpError';
  }
}
