export class AdapterError extends Error {
  constructor(message: string) {
    super(`[Adapter] ${message}`);
    this.name = "AdapterError";
  }
}

export class InitError extends AdapterError {
  constructor(msg: string) { super(msg); }
}

export class OperationError extends AdapterError {
  constructor(msg: string) { super(msg); }
}
